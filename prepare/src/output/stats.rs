use std::collections::HashMap;

pub type CodepointGroups<'a> = HashMap<&'a str, Vec<String>>;

/// информация о данных декомпозиции
pub fn print(filename: &str, index: &[u32], data: &[u64], expansions: &[u32])
{
    println!(
        "\n{}:\n  \
        размер индекса: {}\n  \
        размер блока данных: {}\n  \
        размер дополнительных данных: {}\n  \
        общий размер: {}\n",
        filename,
        index.len(),
        data.len() * 8,
        expansions.len() * 4,
        index.len() + (data.len() * 8) + (expansions.len() * 4),
    );
}

/// группы кодпоинтов
pub fn format_codepoint_groups(stats: CodepointGroups) -> String
{
    let mut output = String::new();

    let mut keys: Vec<&&str> = stats.keys().collect();
    keys.sort();

    for key in keys {
        let data = stats.get(*key).unwrap();

        output.push_str(
            format!(
                "{}\n\n{} \n",
                key,
                data.iter().map(|e| e.as_str()).collect::<String>()
            )
            .as_str(),
        );
    }

    output
}
