use crate::TestHarness;

#[tokio::test]
pub async fn test_signup() {
    let harness = TestHarness::start().await;

    let email = String::from("naruto@gmail.com");
    let pass = String::from("sasuke");
    let user = harness.create_user(&email, &pass).await;
    assert_eq!(user.email, email);
    assert_eq!(user.password_hash, pass);
    
    harness.shutdown().await.unwrap();
}