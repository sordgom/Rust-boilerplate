use zero2prod::config::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise logger
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Read configuration
    let config = get_configuration().expect("Failed to read configuration.");

    // Run the app
    let application = Application::build(config).await?;
    application.run_until_stopped().await?;

    // In case we need to add in more workers
    // let application_task = tokio::spawn(application.run_until_stopped());
    // let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    // tokio::select! {
    //     o = application_task => report_exit("API", o),
    //     o = worker_task =>  report_exit("Background worker", o),
    // };

    Ok(())
}
