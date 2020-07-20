use yew::prelude::*;

pub fn avatar(name: &str) -> Html {
    let firstchar = name.chars().next().unwrap_or('.');
    let hue = (random(firstchar as u32) * 360_f64) as u32;
    html! {
        <div class="container" style=format!("position:relative;background-color:hsl({},100%,50%);height:50px;width:50px;border-radius:50%;",hue)>
            <div style="position:absolute;width:100%;top:50%;transform: translate(0, -50%);margin: 0;" class="has-text-centered">
                {
                    firstchar
                }
            </div>
        </div>
    }
}

pub fn getavatarcolor(name: &str) -> String {
    let firstchar = name.chars().next().unwrap_or('.');
    let hue = (random(firstchar as u32) * 360_f64) as u32;
    format!("hsl({},100%,50%)", hue)
}

pub fn random(seed: u32) -> f64 {
    let x = ((seed + 1) as f64).sin() * 10000_f64;
    x - x.floor()
}
