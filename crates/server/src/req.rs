use serde::{ser::Serialize, de::DeserializeOwned};
use languageserver_types::{TextDocumentIdentifier, Range, Url, Position};
use url_serde;

pub use languageserver_types::{
    request::*, notification::*,
    InitializeResult, PublishDiagnosticsParams,
    DocumentSymbolParams, DocumentSymbolResponse,
    CodeActionParams, ApplyWorkspaceEditParams,
    ExecuteCommandParams,
    WorkspaceSymbolParams,
    TextDocumentPositionParams,
};


pub trait ClientRequest: 'static {
    type Params: DeserializeOwned + Send + 'static;
    type Result: Serialize + Send + 'static;
    const METHOD: &'static str;
}

impl<T> ClientRequest for T
    where T: Request + 'static,
          T::Params: DeserializeOwned + Send + 'static,
          T::Result: Serialize + Send + 'static,
{
    type Params = <T as Request>::Params;
    type Result = <T as Request>::Result;
    const METHOD: &'static str = <T as Request>::METHOD;
}


pub enum SyntaxTree {}

impl Request for SyntaxTree {
    type Params = SyntaxTreeParams;
    type Result = String;
    const METHOD: &'static str = "m/syntaxTree";
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTreeParams {
    pub text_document: TextDocumentIdentifier
}

pub enum ExtendSelection {}

impl Request for ExtendSelection {
    type Params = ExtendSelectionParams;
    type Result = ExtendSelectionResult;
    const METHOD: &'static str = "m/extendSelection";
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtendSelectionParams {
    pub text_document: TextDocumentIdentifier,
    pub selections: Vec<Range>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtendSelectionResult {
    pub selections: Vec<Range>,
}

pub enum FindMatchingBrace {}

impl Request for FindMatchingBrace {
    type Params = FindMatchingBraceParams;
    type Result = Vec<Position>;
    const METHOD: &'static str = "m/findMatchingBrace";
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FindMatchingBraceParams {
    pub text_document: TextDocumentIdentifier,
    pub offsets: Vec<Position>,
}

pub enum PublishDecorations {}

impl Notification for PublishDecorations {
    type Params = PublishDecorationsParams;
    const METHOD: &'static str = "m/publishDecorations";
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishDecorationsParams {
    #[serde(with = "url_serde")]
    pub uri: Url,
    pub decorations: Vec<Decoration>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Decoration {
    pub range: Range,
    pub tag: &'static str
}

pub enum MoveCursor {}

impl Request for MoveCursor {
    type Params = Position;
    type Result = ();
    const METHOD: &'static str = "m/moveCursor";
}
