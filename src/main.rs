use std::{env, fs, error::Error, io};

use fs_extra::dir::CopyOptions;
use html_minifier::HTMLMinifier;
use whitman::constants::VALID_ACTIONS;
use whitman::state::State;

use serde_json::json;
use handlebars::{Handlebars, Helper, Context, RenderContext, Output, RenderError};

fn file_href(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // get parameter from helper or throw an error
    let param = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or_else(|| {
            RenderError::new(
                "Param 0 with u64 type is required for rank helper.",
            )
        })? as &str;
    
    let output = String::from(param);

    out.write(output.replace("file:./", "").replace(".org", ".html").as_ref())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Collect all command line arguments.
    let args: Vec<String> = env::args().collect();
    // Get the first argument; i.e., the "action."
    let action = &args[1];
    // Ensure the action is a valid one.
    if !VALID_ACTIONS.contains(&action.as_str()) {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::InvalidData, "Invalid action")
            )
        )
    }

    // Define the whitman state
    let mut state = State::new(&args);
    // Ensure a config file is present in the provided directory.
    if state.config.is_none() {
        panic!("Unable to to find whitman config in the provided directory");
    }
    // Generate templates + index for the current state.
    state.init()?;
    
    let org_map = &state.index.org_map;
    let template_map = &state.templates;
    let template_dir = &state.get_template_dir()?;
    let output_dir = &state.get_output_dir()?;
    let assets_dir = &state.get_assets_dir()?;
    let partials_dir = template_dir.join("partials");
    let partial = partials_dir.join("tree_render.hbs").to_str().unwrap().to_owned();

    for record in &state.index.records {
        let mut minifier = HTMLMinifier::new();
        let mut reg = Handlebars::new();
        let template = record.template(template_map)?;
        let outline = record.outline(org_map)?;

        let contents = fs::read_to_string(template.path.to_str().unwrap()).unwrap();
        reg.register_helper("file_href", Box::new(file_href));
        reg.register_template_file(
            "tree_render",
            &partial, 
        )?;

        println!("{}\n\n", json!(&outline));

        println!("{}", record.file_name);
        println!("{}", record.path.as_os_str().to_str().unwrap());
        
        let base_path = record.path.clone();
        let prefixless = base_path.strip_prefix(&state.dir)?;
        let mut target_path = output_dir.join(prefixless);

        fs::create_dir_all(&target_path.parent().unwrap())?;
        target_path.set_extension("html");

        let raw_html = reg.render_template(&contents, &outline)?;
        // Temporarily disabling minifaction because it's stripping out characters.
        // FIXME: Replace minifier with something better
        // minifier.digest(&raw_html)?;

        fs::write(&target_path, minifier.get_html()).expect("Unable to write file");

        let mut options = CopyOptions::new();
        options.overwrite = true;
        fs_extra::dir::copy(&assets_dir, &output_dir, &options)?;
    }

    Ok(())
}
