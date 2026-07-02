use genanki_rs::{Deck, Error, Field, Model, Note, Package, Template};
use rand::random_range;

fn random_id() -> i64 {
    random_range(1..1_000_000_000)
}

fn main() -> Result<(), Error> {
    let name = "CardMaker";
    let your_lang = "pt_BR";
    let target_lang = "en_US";
    let highlight_color = "red";
    let model_id = random_id();
    let deck_id = random_id();
    let fields = vec![
        Field::new(&format!("🔤 Text ({target_lang})")),
        Field::new(&format!("🔄 Translation ({your_lang})")),
        Field::new(&format!("❓ New word ({target_lang})")),
        Field::new("🗣️ Pronunciation"),
    ];

    let activate_template = Template::new("Activate")
        .qfmt(
            &include_str!("../cardtypes/Active/Front.html")
                .replace("target_lang", target_lang)
                .replace("your_lang", your_lang)
                .replace("highlight_color", highlight_color),
        )
        .afmt(
            &include_str!("../cardtypes/Active/Back.html")
                .replace("target_lang", target_lang)
                .replace("your_lang", your_lang)
                .replace("highlight_color", highlight_color),
        );

    let passive_template = Template::new("Passive")
        .qfmt(
            &include_str!("../cardtypes/Passive/Front.html")
                .replace("target_lang", target_lang)
                .replace("your_lang", your_lang)
                .replace("highlight_color", highlight_color),
        )
        .afmt(
            &include_str!("../cardtypes/Passive/Back.html")
                .replace("target_lang", target_lang)
                .replace("your_lang", your_lang)
                .replace("highlight_color", highlight_color),
        );

    let writing_template = Template::new("Writing")
        .qfmt(
            &include_str!("../cardtypes/Writing/Front.html")
                .replace("target_lang", target_lang)
                .replace("your_lang", your_lang)
                .replace("highlight_color", highlight_color),
        )
        .afmt(
            &include_str!("../cardtypes/Writing/Back.html")
                .replace("target_lang", target_lang)
                .replace("your_lang", your_lang)
                .replace("highlight_color", highlight_color),
        );

    let model = Model::new(
        model_id,
        &name,
        fields,
        vec![activate_template, passive_template, writing_template],
    );
    let mut deck = Deck::new(deck_id, name, "");
    deck.add_note(Note::new(
        model,
        vec!["Hello world", "Olá mundo", "World", "🗣️ Pronunciation"],
    )?);
    let mut package = Package::new(vec![deck], vec![])?;
    package.write_to_file("output.apkg")?;
    Ok(())
}
