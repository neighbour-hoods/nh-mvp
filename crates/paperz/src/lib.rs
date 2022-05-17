use hdk::prelude::*;

use common::SensemakerEntry;

mod util;

pub const PAPER_TAG: &str = "paperz_paper";
pub const ANN_TAG: &str = "annotationz";

pub const PAPER_PATH: &str = "widget.paperz"; //.${entry_hash} => "sm_data"

entry_defs![
    Path::entry_def(),
    Paper::entry_def(),
    Annotation::entry_def(),
    SensemakerEntry::entry_def()
];

#[hdk_entry]
pub struct Paper {
    // must include extension
    pub filename: String,
    // encoded file bytes payload
    // getting an error here on get_paperz. Deserialize("invalid type: byte array, expected u8")
    pub blob_str: String,
}

#[hdk_entry]
pub struct Annotation {
    pub paper_ref: EntryHash, // should this be a HeaderHash? probably
    pub page_num: u64,
    pub paragraph_num: u64,
    pub what_it_says: String,
    pub what_it_should_say: String,

}

fn paper_anchor() -> ExternResult<EntryHash> {
    anchor("paperz".into(), "".into())
}

#[hdk_extern]
fn upload_paper(paper: Paper) -> ExternResult<HeaderHash> {
    debug!(
        "upload_paper: received input of length {}",
        paper.blob_str.len()
    );

    let paper_hh = create_entry(&paper)?;
    let paper_eh = hash_entry(&paper)?;
    create_link(paper_anchor()?, paper_eh, LinkTag::new(PAPER_TAG))?;

    Ok(paper_hh)
}

#[hdk_extern]
fn get_all_papers(_: ()) -> ExternResult<Vec<(EntryHash, Paper)>> {
    debug!("Getting all paperz...");
    let paper_entry_links = get_links(paper_anchor()?, Some(LinkTag::new(PAPER_TAG)))?;
    let mut paperz: Vec<(EntryHash, Paper)> = Vec::new();
    for lnk in paper_entry_links {
        let res: ExternResult<(EntryHash, Paper)> = {
            let paper_eh = lnk.target;
            let (paper, _hh) =
                util::try_get_and_convert_with_hh(paper_eh.clone(), GetOptions::content())?;
            Ok((paper_eh, paper))
        };

        match res {
            Ok(tup) => paperz.push(tup),
            Err(err) => debug!("err in fetching Paper: {}", err),
        }
    }
    Ok(paperz)
}

fn annotation_anchor() -> ExternResult<EntryHash> {
    anchor(ANN_TAG.into(), "".into())
}

#[hdk_extern]
fn get_annotations_for_paper(paper_entry_hash: EntryHash) -> ExternResult<Vec<(EntryHash, Annotation)>> {
    debug!("Getting annotations");
    let mut annotations: Vec<(EntryHash, Annotation)> = Vec::new();
    debug!("Created empty vector");
    for link in get_links(paper_entry_hash, Some(LinkTag::new(ANN_TAG)))? {
        debug!("Here is a links: {:?}", link);
        let annotation_entry_hash = link.target;
        match util::try_get_and_convert(
            annotation_entry_hash.clone(), 
 GetOptions::content()) 
        {
            Ok(annotation) => {
                debug!("Annotation: {:?}", annotation);
                annotations.push((annotation_entry_hash, annotation));
            }
            Err(err) => {
                error!("get_annotations_for_paper: err: {}", err);
            }
        }
    }
    Ok(annotations)
}

#[hdk_extern]
fn create_annotation(annotation: Annotation) -> ExternResult<(EntryHash, HeaderHash)> {

  let annotation_headerhash = create_entry(&annotation)?;
  let annotation_entryhash = hash_entry(&annotation)?;
  create_link(annotation_anchor()?, annotation_entryhash.clone(), LinkTag::new(ANN_TAG))?;
  create_link(annotation.paper_ref, annotation_entryhash.clone(), LinkTag::new(ANN_TAG))?;

  // this is a write interface between a widget and the sensemaker hub
  call(
    None, // todo: get hub cell
    "hub".into(), 
        "create_sensemaker_entry".into(), 
  None, 
  annotation_entryhash.clone())?;

  Ok((annotation_entryhash, annotation_headerhash))
}

/**
* What is a Vec of (EH, SE) tuples?
*/
#[hdk_extern]
fn get_state_machine_data(
    (target_eh, opt_label): (EntryHash, Option<String>),
) -> ExternResult<Vec<(EntryHash, SensemakerEntry)>> {
    
    match call(    
        None, // todo: get hub cell
        "hub".into(), 
        "get_state_machine_data".into(), 
        None, 
        (target_eh, opt_label))? {
            ZomeCallResponse::Ok(data) => {
                return Ok(data.decode()?);
            },
            _ => todo!(),
        }
}
