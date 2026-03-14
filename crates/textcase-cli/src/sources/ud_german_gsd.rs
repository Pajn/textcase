use super::{SourceRecord, parse_conllu_records};

pub fn parse(bytes: &[u8]) -> Result<Vec<SourceRecord>, Box<dyn std::error::Error>> {
    parse_conllu_records(bytes)
}

pub fn sample() -> Vec<u8> {
    b"# sent_id = 1
1	berlin	Berlin	PROPN	_	_	0	root	_	_
2	ist	sein	AUX	_	_	1	cop	_	_
3	schon	schon	ADV	_	_	1	advmod	_	_
4	gut	gut	ADJ	_	_	1	xcomp	_	_
"
    .to_vec()
}
