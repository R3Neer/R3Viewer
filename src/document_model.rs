//! Closed R3MDv2 records. Only a fully validated candidate becomes Document.
use crate::{
    document_budget::{ActorArena, ArenaVec},
    document_source::DocumentSource,
};
use std::{fmt, sync::Arc};
pub const NONE: u32 = u32::MAX;
pub const RECORD_LIMIT: usize = 1_000_000;
pub const DEPTH_LIMIT: u32 = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Profile {
    Core = 0,
    Extended = 1,
}
impl Profile {
    pub fn from_word(word: u32) -> Result<Self> {
        match word {
            0 => Ok(Self::Core),
            1 => Ok(Self::Extended),
            _ => Err(Error::new(ErrorKind::InvalidContext)),
        }
    }
    pub fn option_bits(self) -> u32 {
        match self {
            Self::Core => 0,
            Self::Extended => 35998,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Parse,
    ArenaLimit,
    DepthLimit,
    RecordLimit,
    InvalidContext,
    Allocation,
    Internal,
    SourceLimit,
    InvalidIpc,
    Cancelled,
    Timeout,
    WorkerExited,
    Containment,
    UnsupportedPlatform,
    PendingCleanup,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub table: u32,
    pub id: u32,
    pub source_offset: u32,
}
impl Error {
    pub const fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            table: 0,
            id: NONE,
            source_offset: NONE,
        }
    }
    pub const fn at(mut self, table: u32, id: u32, source_offset: u32) -> Self {
        self.table = table;
        self.id = id;
        self.source_offset = source_offset;
        self
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "documento Markdown: {:?} (tabla {}, id {})",
            self.kind, self.table, self.id
        )
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
macro_rules! record {
    ($name:ident,$n:literal,$($field:ident:$index:literal),+) => {
        #[derive(Clone,Copy,Debug,PartialEq,Eq)]
        #[repr(transparent)]
        pub struct $name(pub(crate) [u32;$n]);
        impl $name {
            pub fn words(&self)->&[u32;$n] {&self.0}
            $(pub fn $field(&self)->u32 {self.0[$index]})+
            pub(crate) const fn from_words(words:[u32;$n])->Self {Self(words)}
        }
        const _:()=assert!(std::mem::size_of::<$name>()==$n*4 && std::mem::align_of::<$name>()==4);
    }
}
record!(Block,16,id:0,parent:1,first_item:2,item_count:3,domain:4,kind:5,flags:6,aux0:7,aux1:8,aux2:9,aux3:10,origin:11);
record!(Domain,6,id:0,kind:1,owner:2,first_run:3,run_count:4);
record!(Item,4,parent_block:0,ordinal:1,kind:2,target:3);
record!(Run,16,id:0,domain:1,ordinal:2,kind:3,styles:4,text:5,origin:6,link:7,resource:8,aux0:9,aux1:10);
record!(Span,10,id:0,domain:1,parent_span:2,start_ordinal:3,end_ordinal:4,kind:5,link:6,origin:7);
record!(Link,16,id:0,kind:1,flags:2,target_decoded:3,raw_token:4,title_decoded:5,reference_label:6,label_domain:7,label_start:8,label_count:9,origin:10,target_origin:11);
record!(Resource,20,id:0,kind:1,flags:2,input_decoded:3,raw_token:4,title_decoded:5,origin:6,alt_domain:7,width_kind:8,width_value:9,height_kind:10,height_value:11,intent:12,owner_run:13,aux0:14,aux1:15);
record!(TextRef,4,kind:0,start:1,end:2,origin:3);
record!(Origin,4,relation:0,start:1,end:2,reserved:3);
record!(Attribute,6,owner_block:0,ordinal:1,kind:2,key:3,value:4,origin:5);
pub(crate) struct Candidate {
    pub source: Arc<DocumentSource>,
    pub profile: Profile,
    pub max_depth: u32,
    pub arena: ActorArena,
    pub blocks: ArenaVec<Block>,
    pub domains: ArenaVec<Domain>,
    pub items: ArenaVec<Item>,
    pub runs: ArenaVec<Run>,
    pub spans: ArenaVec<Span>,
    pub links: ArenaVec<Link>,
    pub resources: ArenaVec<Resource>,
    pub texts: ArenaVec<TextRef>,
    pub origins: ArenaVec<Origin>,
    pub attributes: ArenaVec<Attribute>,
    pub decoded: ArenaVec<u8>,
}
impl Candidate {
    pub fn empty(source: Arc<DocumentSource>, profile: Profile, arena: ActorArena) -> Self {
        Self {
            source,
            profile,
            max_depth: 0,
            blocks: ArenaVec::new(arena.clone()),
            domains: ArenaVec::new(arena.clone()),
            items: ArenaVec::new(arena.clone()),
            runs: ArenaVec::new(arena.clone()),
            spans: ArenaVec::new(arena.clone()),
            links: ArenaVec::new(arena.clone()),
            resources: ArenaVec::new(arena.clone()),
            texts: ArenaVec::new(arena.clone()),
            origins: ArenaVec::new(arena.clone()),
            attributes: ArenaVec::new(arena.clone()),
            decoded: ArenaVec::new(arena.clone()),
            arena,
        }
    }
    pub fn counts(&self) -> [u32; 11] {
        [
            self.blocks.len() as u32,
            self.domains.len() as u32,
            self.items.len() as u32,
            self.runs.len() as u32,
            self.spans.len() as u32,
            self.links.len() as u32,
            self.resources.len() as u32,
            self.texts.len() as u32,
            self.origins.len() as u32,
            self.attributes.len() as u32,
            self.decoded.len() as u32,
        ]
    }
    pub fn record_count(&self) -> usize {
        self.counts()[..10].iter().map(|&n| n as usize).sum()
    }
    pub fn text(&self, id: u32) -> Result<&str> {
        let t = self
            .texts
            .get(id as usize)
            .ok_or_else(|| Error::new(ErrorKind::InvalidContext).at(8, id, NONE))?;
        match t.kind() {
            0 => self.source.range(t.start(), t.end()),
            1 => std::str::from_utf8(
                self.decoded
                    .get(t.start() as usize..t.end() as usize)
                    .ok_or_else(|| Error::new(ErrorKind::InvalidContext).at(8, id, NONE))?,
            )
            .map_err(|_| Error::new(ErrorKind::InvalidContext).at(8, id, NONE)),
            _ => Err(Error::new(ErrorKind::InvalidContext).at(8, id, NONE)),
        }
    }
}
pub struct Document {
    inner: Candidate,
}
impl Document {
    pub(crate) fn reserve_consumer(&self, bytes: usize) -> Result<crate::document_budget::Lease> {
        self.inner.arena.reserve(bytes)
    }
    pub(crate) fn from_validated(proof: crate::document_validate::ValidatedCandidate) -> Self {
        Self {
            inner: proof.into_inner(),
        }
    }
    pub fn source(&self) -> &DocumentSource {
        &self.inner.source
    }
    pub fn profile(&self) -> Profile {
        self.inner.profile
    }
    pub fn counts(&self) -> [u32; 11] {
        self.inner.counts()
    }
    pub fn max_depth(&self) -> u32 {
        self.inner.max_depth
    }
    pub fn text(&self, id: u32) -> Result<&str> {
        let t = self
            .inner
            .texts
            .get(id as usize)
            .ok_or_else(|| Error::new(ErrorKind::InvalidContext))?;
        match t.kind() {
            0 => self.inner.source.range(t.start(), t.end()),
            1 => {
                // Document can only be committed after the complete pool and every TextRef UTF8 boundary have been validated.
                let pool = unsafe { std::str::from_utf8_unchecked(&self.inner.decoded) };
                pool.get(t.start() as usize..t.end() as usize)
                    .ok_or_else(|| Error::new(ErrorKind::InvalidContext))
            }
            _ => Err(Error::new(ErrorKind::InvalidContext)),
        }
    }
    pub fn blocks(&self) -> &[Block] {
        &self.inner.blocks
    }
    pub fn domains(&self) -> &[Domain] {
        &self.inner.domains
    }
    pub fn items(&self) -> &[Item] {
        &self.inner.items
    }
    pub fn runs(&self) -> &[Run] {
        &self.inner.runs
    }
    pub fn spans(&self) -> &[Span] {
        &self.inner.spans
    }
    pub fn links(&self) -> &[Link] {
        &self.inner.links
    }
    pub fn resources(&self) -> &[Resource] {
        &self.inner.resources
    }
    pub fn text_refs(&self) -> &[TextRef] {
        &self.inner.texts
    }
    pub fn origins(&self) -> &[Origin] {
        &self.inner.origins
    }
    pub fn attributes(&self) -> &[Attribute] {
        &self.inner.attributes
    }
    pub fn arena_used(&self) -> usize {
        self.inner.arena.used()
    }
    pub fn arena_peak(&self) -> usize {
        self.inner.arena.peak()
    }
}
