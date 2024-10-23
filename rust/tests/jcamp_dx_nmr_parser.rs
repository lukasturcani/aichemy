use aichemy::nmr::io::jcamp_dx::{self, Value};

#[test]
fn test_parse() {
    let items = jcamp_dx::parse(
        b"
        ##TITLE= diff
        ##JCAMPDX= 5.0         $$ Bruker NMR JCAMP-DX V1.0
        ##DATA TYPE= NMR Spectrum
        ##.OBSERVE FREQUENCY= 100.4
        ##.OBSERVE NUCLEUS= ^13C
        ##SPECTROMETER/DATA SYSTEM= JEOL GX 400
        $$ Bruker specific parameters
        $$ --------------------------
        ##$AQ_mod= 1
        ##$AUNM= <au_zgsino>
        ##$BF1= 100.4
        ##$CPDPRGB= <waltz16>
        ##$D= (0..3)
        0 1 2 3
        ##$DBPNAM0= <>
        ##$DECNUC= <1H>
        ##$IN= (0..4)
        0.001 0.002 0.003
        0.004 0.005
        ##MINY= -27593530
        ##XYDATA=(X++(Y..Y))
                   16383       2259260      -5242968      -7176216      -1616072
                    7915       3754660       -142736        -85762      -2471282
        ##END=",
    )
    .unwrap();
    assert_eq!(
        {
            let mut keys = items.keys().collect::<Vec<_>>();
            keys.sort();
            keys
        },
        {
            let mut expected = [
                "TITLE",
                "JCAMPDX",
                "DATATYPE",
                ".OBSERVEFREQUENCY",
                ".OBSERVENUCLEUS",
                "SPECTROMETERDATASYSTEM",
                "$AQMOD",
                "$AUNM",
                "$BF1",
                "$CPDPRGB",
                "$D",
                "$DBPNAM0",
                "$DECNUC",
                "$IN",
                "MINY",
                "XYDATA",
                "END",
            ];
            expected.sort();
            expected
        }
    );
    assert_eq!(items["TITLE"], Value::String("diff".into()));
    assert_eq!(items["JCAMPDX"], Value::Number(5.0));
    assert_eq!(items["DATATYPE"], Value::String("NMR Spectrum".into()));
    assert_eq!(items[".OBSERVEFREQUENCY"], Value::Number(100.4));
    assert_eq!(items[".OBSERVENUCLEUS"], Value::String("^13C".into()));
    assert_eq!(
        items["SPECTROMETERDATASYSTEM"],
        Value::String("JEOL GX 400".into())
    );
    assert_eq!(items["$AQMOD"], Value::Number(1.));
    assert_eq!(items["$AUNM"], Value::String("au_zgsino".into()));
    assert_eq!(items["$BF1"], Value::Number(100.4));
    assert_eq!(items["$CPDPRGB"], Value::String("waltz16".into()));
    assert_eq!(items["$D"], Value::NumberArray(vec![0., 1., 2., 3.]));
    assert_eq!(items["$DBPNAM0"], Value::String("".into()));
    assert_eq!(items["$DECNUC"], Value::String("1H".into()));
    assert_eq!(
        items["$IN"],
        Value::NumberArray(vec![0.001, 0.002, 0.003, 0.004, 0.005])
    );
    assert_eq!(items["MINY"], Value::Number(-27593530.));
    assert_eq!(
        items["XYDATA"],
        Value::NumberArray(vec![
            2259260., -5242968., -7176216., -1616072., 3754660., -142736., -85762., -2471282.
        ])
    );
}
