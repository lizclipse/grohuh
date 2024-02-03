use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{self, Ws},
    Surreal,
};
use ulid::Ulid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Surreal::new::<Ws>("localhost:37002").await?;
    db.use_ns("dev").use_db("grohuh").await?;

    let mut mqttoptions = MqttOptions::new("grohuh", "localhost", 37_000);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("energy/growatt", QoS::ExactlyOnce).await?;

    loop {
        let notification = eventloop.poll().await;
        if let Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg))) = notification {
            match ingest_msg(&db, msg).await {
                Ok(_) => {
                    println!("Ingested");
                }
                Err(err) => {
                    eprintln!("Failed to ingest = {:#?}", err);
                }
            }
        }
    }
}

async fn ingest_msg(db: &Surreal<ws::Client>, msg: rumqttc::Publish) -> anyhow::Result<()> {
    println!("Received = {:#?}", msg);
    let msg: GrowattMessage = serde_json::from_slice(&msg.payload)?;
    let msg: DataRecord = msg.into();
    let _: Option<DataRecord> = db
        .create(("data", Ulid::new().to_string().to_ascii_lowercase()))
        .content(msg)
        .await?;

    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Readings {
    #[serde(rename = "epv1today")]
    epv1today: i64,
    #[serde(rename = "epv1total")]
    epv1total: i64,
    #[serde(rename = "epv2today")]
    epv2today: i64,
    #[serde(rename = "epv2total")]
    epv2total: i64,
    #[serde(rename = "epvtotal")]
    epvtotal: i64,
    #[serde(rename = "pv1current")]
    pv1current: i64,
    #[serde(rename = "pv1voltage")]
    pv1voltage: i64,
    #[serde(rename = "pv1watt")]
    pv1watt: i64,
    #[serde(rename = "pv2current")]
    pv2current: i64,
    #[serde(rename = "pv2voltage")]
    pv2voltage: i64,
    #[serde(rename = "pv2watt")]
    pv2watt: i64,
    #[serde(rename = "pvenergytoday")]
    pvenergytoday: i64,
    #[serde(rename = "pvenergytotal")]
    pvenergytotal: i64,
    #[serde(rename = "pvfrequentie")]
    pvfrequentie: i64,
    #[serde(rename = "pvgridcurrent")]
    pvgridcurrent: i64,
    #[serde(rename = "pvgridcurrent2")]
    pvgridcurrent2: i64,
    #[serde(rename = "pvgridcurrent3")]
    pvgridcurrent3: i64,
    #[serde(rename = "pvgridpower")]
    pvgridpower: i64,
    #[serde(rename = "pvgridpower2")]
    pvgridpower2: i64,
    #[serde(rename = "pvgridpower3")]
    pvgridpower3: i64,
    #[serde(rename = "pvgridvoltage")]
    pvgridvoltage: i64,
    #[serde(rename = "pvgridvoltage2")]
    pvgridvoltage2: i64,
    #[serde(rename = "pvgridvoltage3")]
    pvgridvoltage3: i64,
    #[serde(rename = "pvipmtemperature")]
    pvipmtemperature: i64,
    #[serde(rename = "pvpowerin")]
    pvpowerin: i64,
    #[serde(rename = "pvpowerout")]
    pvpowerout: i64,
    #[serde(rename = "pvstatus")]
    pvstatus: i64,
    #[serde(rename = "pvtemperature")]
    pvtemperature: i64,
    #[serde(rename = "recortype1")]
    recortype1: i64,
    #[serde(rename = "recortype2")]
    recortype2: i64,
    #[serde(rename = "totworktime")]
    totworktime: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GrowattMessage {
    buffered: String,
    device: String,
    time: String,
    values: Readings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataRecord {
    buffered: String,
    device: String,
    time: String,
    #[serde(flatten)]
    values: Readings,
}

impl From<GrowattMessage> for DataRecord {
    fn from(value: GrowattMessage) -> Self {
        Self {
            buffered: value.buffered,
            device: value.device,
            time: value.time,
            values: value.values,
        }
    }
}
