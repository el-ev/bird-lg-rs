use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TableRow {
    pub cells: Vec<Html>,
    pub on_click: Option<Callback<MouseEvent>>,
}

#[derive(Properties, PartialEq)]
pub struct DataTableProps {
    pub headers: Vec<String>,
    pub rows: Vec<TableRow>,
}

#[function_component(DataTable)]
pub fn data_table(props: &DataTableProps) -> Html {
    html! {
        <table class="data-table">
            <thead>
                <tr>
                    { for props.headers.iter().map(|header| html! { <th>{ header }</th> }) }
                </tr>
            </thead>
            <tbody>
                { for props.rows.iter().map(|row| {
                    let on_click = row.on_click.clone().unwrap_or_default();
                    html! {
                        <tr class={
                            if row.on_click.is_some() {
                                "clickable-row"
                            } else {
                                ""
                            }
                        } onclick={on_click}>
                            { for row.cells.iter().map(|cell| html! { <td>{ cell.clone() }</td> }) }
                        </tr>
                    }
                }) }
            </tbody>
        </table>
    }
}
