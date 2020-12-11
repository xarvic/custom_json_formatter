use serde_json::Serializer;
use custom_json_formatter::OpenStructures;
use serde::Serialize;

#[derive(Serialize)]
struct A<'a>{
    name: &'a str,
    childs: Vec<A<'a>>,
}



fn main() -> Result<(), serde_json::Error> {
    let value = A{
        name: "layer1",
        childs: vec![
            A{
                name: "layer2",
                childs: vec![
                    A{
                        name: "layer3",
                        childs: vec![
                        ]
                    },
                    A{
                        name: "drgd",
                        childs: vec![
                        ]
                    },
                    A{
                        name: "hi",
                        childs: vec![
                        ]
                    },
                ]
            },
            A{
                name: "layer2...",
                childs: vec![
                    A{
                        name: "lafrdgyer4",
                        childs: vec![
                        ]
                    },
                    A{
                        name: "drefsgd",
                        childs: vec![
                        ]
                    },
                    A{
                        name: "hfesei",
                        childs: vec![
                        ]
                    },
                ]
            },
        ]
    };

    println!("Pretty:");
    println!("{}", serde_json::to_string_pretty(&value)?);


    println!("Fold after:");
    let mut target = Vec::<u8>::new();
    let mut serializer = Serializer::with_formatter(
        &mut target,
        OpenStructures::new("  ", 5)
    );
    value.serialize(&mut serializer);
    println!("{}", String::from_utf8(target).unwrap());

    Ok(())
}