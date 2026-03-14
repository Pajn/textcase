use super::SourceRecord;

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    let text = std::str::from_utf8(bytes)?;
    let mut records = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() != 10 {
            return Err(format!("expected 10 CoNLL-U columns, got {}", columns.len()).into());
        }

        let token_id = columns[0].trim();
        if token_id.contains('-') || token_id.contains('.') {
            continue;
        }

        let form = columns[1].trim();
        let lemma = columns[2].trim();
        let upos = columns[3].trim();
        let feats = columns[5].trim();
        let deprel = columns[7].trim();
        let misc = columns[9].trim();

        if !matches!(upos, "NOUN" | "PROPN") {
            continue;
        }

        let canonical = if upos == "PROPN" {
            form.to_string()
        } else {
            capitalize_noun_lemma(lemma)
        };

        let mut aliases = vec![form.to_string()];
        if lemma != form {
            aliases.push(lemma.to_string());
        }

        records.push(SourceRecord {
            canonical,
            aliases,
            score: german_score(upos, feats, deprel, misc),
        });
    }
    Ok(records)
}

pub fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let records = parse(bytes)?;
    if records.is_empty() {
        return Err("UD German GSD payload did not yield noun candidates".into());
    }
    Ok(())
}

pub fn sample() -> Vec<u8> {
    b"# sent_id = 1
1	Berlin	Berlin	PROPN	NE	Case=Nom|Number=Sing	0	root	_	_
2	ist	sein	AUX	_	_	1	cop	_	_
3	schon	schon	ADV	_	_	1	advmod	_	_
4	gut	gut	ADJ	_	_	1	xcomp	_	_
5	Probleme	Problem	NOUN	NN	Case=Nom|Gender=Neut|Number=Plur	1	conj	_	_
"
    .to_vec()
}

fn german_score(upos: &str, feats: &str, deprel: &str, misc: &str) -> f32 {
    let mut score = match upos {
        "PROPN" => 3.0,
        "NOUN" => 2.0,
        _ => 0.0,
    };
    if feats.contains("Case=Nom") {
        score += 0.4;
    }
    if feats.contains("Number=Sing") {
        score += 0.1;
    }
    if feats.contains("Gender=") {
        score += 0.1;
    }
    if matches!(deprel, "nsubj" | "obj" | "obl" | "nmod" | "root" | "appos") {
        score += 0.3;
    }
    if misc.contains("NE=") {
        score += 0.5;
    }
    score
}

fn capitalize_noun_lemma(lemma: &str) -> String {
    let mut chars = lemma.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = first.to_uppercase().collect::<String>();
    out.push_str(chars.as_str());
    out
}

#[cfg(test)]
mod tests {
    use super::{parse, sample};

    #[test]
    fn parses_only_nouns_and_propns() {
        let records = parse(&sample()).expect("parse sample");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].canonical, "Berlin");
        assert_eq!(records[1].canonical, "Problem");
    }

    #[test]
    fn scores_proper_nouns_above_common_nouns() {
        let records = parse(&sample()).expect("parse sample");
        assert!(records[0].score > records[1].score);
    }
}
