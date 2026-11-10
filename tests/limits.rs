use std::fs;
use std::path::Path;

#[test]
fn sources_do_not_reference_network_clients() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut buf = String::new();
    read_tree(&root, &mut buf);
    for banned in [
        "reqwest",
        "hyper::",
        "TcpStream",
        "UdpSocket",
        "std::net",
        "ureq",
    ] {
        assert!(
            !buf.contains(banned),
            "src mentions {banned}, but this tool does not use the network"
        );
    }
}

fn read_tree(dir: &Path, buf: &mut String) {
    for entry in fs::read_dir(dir).expect("read src") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            read_tree(&path, buf);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            buf.push_str(&fs::read_to_string(&path).expect("read rs"));
        }
    }
}
