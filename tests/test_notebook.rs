use kernel_sidecar_rs::notebook::Notebook;

#[test]
fn test_notebook_structure() {
    // read tests/demo_notebook.ipynb
    let content = std::fs::read_to_string("tests/demo_notebook.ipynb").unwrap();

    let nb: Notebook = serde_json::from_str(&content).unwrap();
    println!("nb: {:?}", nb);

    // Serialize the Notebook back to JSON
    let serialized: String = serde_json::to_string(&nb).unwrap();

    let nb2: Notebook = serde_json::from_str(&serialized).unwrap();
    assert_eq!(nb, nb2);
}
