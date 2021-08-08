use mysql_rent::Rent;

#[tokio::test]
async fn should_work_with_no_params() {
    let _sut = Rent::new().await.unwrap();
}

#[tokio::test]
async fn should_work_with_options() {
    let _sut = Rent::builder()
        .container_name("other-container")
        .database("contacts")
        .local_port(3307u16)
        .root_password("chupacabra111")
        .rent()
        .await
        .unwrap();
}
