use mysql_rent::Rent;

#[tokio::test]
async fn should_work_with_no_params() {
    let sut = Rent::new().await.unwrap();
    println!("connection URL: {}", sut.mysql_url());
}

#[tokio::test]
async fn should_work_with_options() {
    let sut = Rent::builder()
        .container_name("other-container")
        .database("contacts")
        .local_port(3307u16)
        .root_password("chupacabra111")
        .rent()
        .await
        .unwrap();
    println!("connection URL: {}", sut.mysql_url());
}
