use yew::{classes, function_component, html, Html};


#[function_component(CompetitionCard)]
pub fn competition_card() -> Html {
    html! {
        <div class={classes!("event-card")}>
           <div class={"event-header"}>
                <h4>{"Соревнования по борьбе в сфере политической гимнастики для партийных работников, 1991 год. "}</h4>
                <a>{"перейти"}</a>
            </div>
            <div class={classes!("event-data")}>
                <div>
                    <div class={classes!("card")}>
                        <div>
                            <ul>
                                <li>{"Даты проведения: "}<span class={"marked"}>{"24.09.24"}</span></li>
                                <li>{"Регистрация до: "}<span class={"marked"}>{"21.09.24"}</span>{": "}<span class={"marked-ok"}>{"открыто"}</span></li>
                                <li>{"Даты проведения: "}<span class={"marked"}>{"24.09.24"}</span></li>
                            </ul>
                        </div>
                    </div>
                    <div class={classes!("card")}>
                        <div>
                            <ul>
                                <li>{"Трансляция: "}<span class={"marked-err"}>{"нет"}</span></li>
                                <li>{"Организатор: "}<span class={"marked"}>{"ЦК ВКПб"}</span></li>
                                <li>{"Результаты: "}<span class={"marked-err"}>{"нет"}</span></li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
