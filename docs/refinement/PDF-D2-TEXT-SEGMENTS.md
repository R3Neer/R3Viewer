# D2 TextChunk Read adapter implementation mechanism

Private proposal only; no adapter source from rejected command exists. Auto-review classified new type/layout as requiring prior contrast. Canonical PDF-P1-WIRE R5 explicitly requires streamR3UTv3 fragmented arbitrarily (including headers), existing incremental validator adapted to Read segments and no innerLength capture. This is representation within that policy, no op/profile/quotas/clocks change. Current source/bin/fixtures snapshot457058EBCB47D131349EA11F5EB5BA2B0A98ECE0D9886C5688151A4EC3800556 remains immutable.

Primary UTF16R2 source identities rechecked against MANIFEST E3FA1AED161E589A448E250ED12C01E0952B2B2CA67618353A76824A9DA9D435: wire_gate.rs18979B SHA66339D6BB98DF39C727D17031DF7AAC0A2C1511C4461DFF46F641BDEB599CD4E, utf16_gate.rs18665B SHA5CDDCAE44B35565C73695D708AD390A278D0F37C2FC32CBF1853612E9077A9D3, lib.rs32002B SHAA8159C123793B30F2FBD00BFEDCFC0B01C186348B28D34C93D3F1B13B9053140. No historical files changed or spikes repeated. The adapter is not a port/acceptance of old helper Python, native API, or semantic validator yet.

## Exact proposed type/factory

TextSegments<'a,R:Read> private fields decoder:&'a mut OuterResponse, reader:&'a mut R, scratch:&'a mut FrameScratch, position:usize,end:usize,eof:bool,metadata:Lease. It has no Vec, Arc allocation, source copy or innerLength storage. No Clone/Default/raw constructor. new(decoder,reader,scratch)->Result<Self,PdfCause> valid only directly after actual TextDescriptor: decoder private State::Payload{kind15,received0,target>0}. Checks scratch body/fragment SAME original source Actor BEFORE reserve. Admits actual sizeof TextSegments on SAME source Actor BEFORE fields/borrows stored; no new limit. ABI/layout inventory and exactsource/bin before output. Empty innerLength cannot represent R3UT header and does not create an adapter/semantic Layer; outer decoder may separately validate zero transport bytes but semantic gate rejects.

## Real Read implementation

Read::read(emptyOut) returns0 according Read API, not EOF witness. Otherwise, while no unread body remains, asks SAME decoder.read_next on original reader+scratch. For actual OuterPiece::Chunk(kind15), stores payload indices8..8+len in existing scratch.body; copies only requested prefix into the caller buffer. It never accumulates all innerLength, copies into a second fragment Vec, grows scratch, clones body, or captures raw stream. Arbitrary inner header/row reads naturally span outer chunks. Outer semantic offsets/targets are still validated by decoder; adapter cannot relabel unknown kind/raster/metadata.

After final TextChunk consumed, the next nonempty Read request drives actual Outer End then actual underlying Read EOF. Only OuterPiece::Eof sets adapter.eof and returns0. End alone, declaredlength, dropped adapter, OptionNone or caller flags never produce EOF/Ready. Trailing/Enddigest/count/total/Readerror remain decoder Failed and return IO/InvalidData; no innerEOF on failure. Unexpected piece sets decoder Failed. No IO under registry/Shared/scratch Mutex; eventual real reader wrapper observes writerClosed before first response byte and owns original IO/cancel/deadline. Adapter alone does not execute that wrapper.

No Ready, physical proof, SourceExact, nativeSuccess or full text proof can be minted from this type. UTF16R2 semantic validator integration must retain original mapping/counts/stride/summary/failure checks, paid table/text buffers and changed known capture/layout; that integration is later D2 code, not inherited from a transport PASS. Borrow/lease ends at last adapter Drop; original parser/source/scratch charges remain retained.

## Oracles before affected output

1. new before Begin/descriptor, on metadata/raster, or after partially consumed text rejects without state/Read/alias changes; source foreign scratch rejects before reserve.
2. 0/N-1/exact constructor header reservation computed from new compiled source before runtime; no alias/body allocation; lease returns original baseline on Drop.
3. Read count1 synthetic innerX via original scratch, then drives End and real outer EOF; pointer/capacity unchanged. X remains invalid R3UT, no Layer/Ready/SourceExact.
4. Header32, row64 and scalar40 split across arbitrary outer chunk boundaries are fed to actual incremental inner reader; no innerLength Vec. Valid/invalid semantic expectations remain original UTF16R2, independently reviewed later.
5. End+underlyingEOFerror and trailing bytes cannot yield successful innerEOF; failed decoder cannot be reused. Drop before EOF is not proof; physical owner/charges stay in original capsule.

All original O3/O7/T01-16/P01-26 remain, native/registry/backstop/publication unresolved. This proposal asks review only of adapter mechanism, never whole D2 acceptance.
