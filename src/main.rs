use gluon::ThreadExt;
use serde_state::ser::SerializeState;
use serde_state::de::DeserializeState;

fn serialize_value(value: gluon::vm::Variants, ser_state: &gluon::vm::serialization::SeSeed) -> Box<[u8]> {
    let mut buffer = Vec::new();
    {
        let mut ser = serde_json::Serializer::pretty(&mut buffer);
        value.serialize_state(&mut ser, &ser_state).unwrap();
    }
    buffer.into_boxed_slice()
}

fn deserialize_value(
    de_state: &mut gluon::vm::serialization::DeSeed,
    serialized: &[u8],
) -> gluon::vm::thread::RootedValue<gluon::vm::thread::RootedThread> {
    let mut de = serde_json::Deserializer::from_slice(&serialized);
    gluon::vm::thread::RootedValue::<_>::deserialize_state(de_state, &mut de).unwrap()
}

fn test_program(name: &str) {
    let vm = gluon::new_vm();
    let program_path = format!("gluon/{}.glu", name);
    vm.load_file(&program_path).unwrap();

    let (program, _) = vm
        .run_expr::<gluon::vm::api::OpaqueValue<&gluon::Thread, gluon::vm::api::Hole>>(
            "program",
            "import! gluon.test",
        )
        .unwrap_or_else(|e| {
            println!("{}", e);
            panic!("fail to parse program")
        });
    let variant = program.get_variant();
    let ser_state = gluon::vm::serialization::SeSeed::new();
    let ser_1 = serialize_value(variant.clone(), &ser_state);
    let ser_2 = serialize_value(variant, &ser_state);

    let vm2 = gluon::new_vm();
    let mut ctx = vm2.current_context();
    let mut de_state = gluon::vm::serialization::DeSeed::new(&vm2, &mut ctx);

    println!("Deserializing #1:\n{}", String::from_utf8(ser_1.to_vec()).unwrap());
    deserialize_value(&mut de_state, &ser_1);
    println!("Deserializing #1 successful!");


    println!("Evalulating a random expression...");
    // This deadlocks
    vm2.run_expr::<gluon::vm::api::OpaqueValue<&gluon::Thread, gluon::vm::api::Hole>>(
        "blah",
        "0",
    ).unwrap();
    println!("Done evalulating!");


    println!("Deserializing #2:\n{}", String::from_utf8(ser_2.to_vec()).unwrap());
    deserialize_value(&mut de_state, &ser_2);
    println!("Deserializing #2 successful!");
}

fn main() {
    // panics
    test_program("test");
}
