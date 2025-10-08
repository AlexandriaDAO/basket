use candid::Principal;

fn main() {
    let principals = vec![
        ("ALEX", "ysy5f-2qaaa-aaaap-qkmmq-cai"),
        ("ZERO", "b3d2q-ayaaa-aaaap-qqcfq-cai"),
        ("KONG", "xnjld-hqaaa-aaaar-qah4q-cai"),
        ("BOB", "7pail-xaaaa-aaaas-aabmq-cai"),
    ];

    for (name, principal_str) in principals {
        match Principal::from_text(principal_str) {
            Ok(_) => println!("{}: Valid", name),
            Err(e) => println!("{}: Invalid - {}", name, e),
        }
    }
}
