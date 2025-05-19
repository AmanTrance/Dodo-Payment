use tokio::time;
use tokio_postgres::NoTls;

pub async fn create_postgres_connection()
-> Result<tokio_postgres::Client, tokio_postgres::error::Error> {
    let mut error = None;

    for i in 1..=50 {
        match tokio_postgres::connect("host=localhost port=5432 user=postgres password=postgres dbname=postgres sslmode=disable", NoTls).await {
            Ok (result) => {
                tokio::spawn((async move || {
                    match result.1.await {
                        Ok (_) => (),
                        Err(_) => ()
                    }
                })());

                return Ok (result.0);
            },

            Err (e) =>  {
                if i == 50 {
                    error = Some (e);
                }

                time::sleep(time::Duration::from_secs(5)).await;

                continue;
            }
        }
    }

    Err(error.unwrap())
}
