mod mapper;
mod output;

use nu_plugin::{
    EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand, serve_plugin,
};
use nu_protocol::{
    ByteStream, ByteStreamType, LabeledError, PipelineData, Signals, Signature, Type,
};

struct ViewerPlugin;
struct Encode;

impl Plugin for ViewerPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Encode)]
    }
}

impl PluginCommand for Encode {
    type Plugin = ViewerPlugin;

    fn name(&self) -> &str {
        "r3viewer encode"
    }
    fn description(&self) -> &str {
        "Codifica valores para el transporte privado R3Viewer wire v1"
    }
    fn signature(&self) -> Signature {
        Signature::build(self.name()).input_output_type(Type::Any, Type::Binary)
    }

    fn run(
        &self,
        _plugin: &ViewerPlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let cwd = engine.get_current_dir()?;
        let reader =
            output::OutputReader::new(input, cwd, engine.signals().clone()).map_err(|error| {
                LabeledError::new("No se puede codificar la entrada de R3Viewer")
                    .with_label(error.to_string(), call.head)
            })?;
        // OutputReader owns the invocation's cancellation token. An outer
        // interruptible ByteStream could report EOF before emitting the error.
        Ok(PipelineData::byte_stream(
            ByteStream::read(reader, call.head, Signals::empty(), ByteStreamType::Binary),
            None,
        ))
    }
}

fn main() {
    // stdout belongs exclusively to the Nushell plugin protocol.
    serve_plugin(&ViewerPlugin, MsgPackSerializer);
}
