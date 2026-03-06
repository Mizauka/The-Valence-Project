use serde::Serialize;

/// Unified CLI output envelope shared by tvp_cli and tvp_hrt_cli.
#[derive(Debug, Serialize)]
pub struct Envelope<Subject, Route, Dosage, Events, ResultData> {
    pub schema: &'static str,
    pub engine: &'static str,
    pub command: &'static str,

    pub db_dir: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_kind: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<Route>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosage: Option<Dosage>,

    pub events: Events,
    pub result: ResultData,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl<Subject, Route, Dosage, Events, ResultData>
    Envelope<Subject, Route, Dosage, Events, ResultData>
{
    pub fn new(
        engine: &'static str,
        command: &'static str,
        db_dir: String,
        events: Events,
        result: ResultData,
    ) -> Self {
        Self {
            schema: "tvp.cli.envelope.v1",
            engine,
            command,
            db_dir,
            subject_kind: None,
            subject: None,
            route: None,
            dosage: None,
            events,
            result,
            notes: None,
            warnings: None,
        }
    }
}
