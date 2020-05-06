use gluon::ThreadExt;
use serde_state::ser::SerializeState;

fn serialize_value(value: gluon::vm::Variants) -> Box<[u8]> {
    let mut buffer = Vec::new();
    {
        let mut ser = serde_cbor::Serializer::new(&mut buffer);
        let ser_state = gluon::vm::serialization::SeSeed::new();
        value.serialize_state(&mut ser, &ser_state).unwrap();
    }
    buffer.into_boxed_slice()
}

fn deserialize_value(
    thread: &gluon::RootedThread,
    serialized: &[u8],
) -> gluon::vm::thread::RootedValue<gluon::vm::thread::RootedThread> {
    let mut de = serde_cbor::Deserializer::from_slice(&serialized);
    gluon::vm::serialization::DeSeed::new(thread, &mut thread.current_context())
        .deserialize(&mut de)
        .unwrap()
}

fn test_program(name: &str) {
    let vm = gluon::new_vm();
    let program_path = format!("gluon/{}.glu", name);
    vm.load_file(&program_path).unwrap();

    let (program, _) = vm
        .run_expr::<gluon::vm::api::OpaqueValue<&gluon::Thread, gluon::vm::api::Hole>>(
            "program",
            &format!("import! gluon.{}", name),
        )
        .unwrap_or_else(|e| panic!("fail to parse program: {}", e));
    let variant = program.get_variant();
    let serialized_client = serialize_value(variant);

    let vm2 = gluon::new_vm();
    println!("Deserializing...");
    deserialize_value(&vm2, &serialized_client);
    println!("Deserializing successful!");
}

fn main() {
    test_program("pass");

    // panics
    test_program("fail");
}
