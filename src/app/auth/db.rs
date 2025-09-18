use argon2::password_hash::PasswordHashString;
use tokio_postgres::{Client, Config, Error, NoTls, Row, Statement};

#[derive(Debug)]
pub struct Database {
    client: Client,
    create_user_statement: Statement,
    get_user_statement: Statement,
}
impl Database {
    pub async fn new(cfg: &Config) -> Self {
        let (client, connection) = cfg.connect(NoTls).await.unwrap();
        tokio::spawn(async {
            connection.await.unwrap(); // run the connection on a bg task
        });
        let create_user_statement = client
            .prepare(include_str!("sql/create_user.sql"))
            .await
            .unwrap();
        let get_user_statement = client
            .prepare(include_str!("sql/get_user.sql"))
            .await
            .unwrap();
        Self {
            client,
            create_user_statement,
            get_user_statement,
        }
    }
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: PasswordHashString,
    ) -> Result<(), Error> {
        self.client
            .execute(
                &self.create_user_statement,
                &[&username, &email, &password_hash.as_str()],
            )
            .await
            .map(|_| ())
    }
    pub async fn get_user(&self, email: &str) -> Result<Row, Error> {
        self.client
            .query_one(&self.get_user_statement, &[&email])
            .await
    }
}
