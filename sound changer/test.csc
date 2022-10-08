feature_def
    switch type(consonant, vowel) root

    feature +voice consonant
    switch manner(plosive, nasal, fricative, approximant, affricate, trill, tap) consonant
    switch place(bilabial, labiodental, dental, alveolar, postalveolar, retroflex, palatal, velar, uvular, pharyngeal, glottal) consonant
    feature +lateral (fricative, approximant)
    feature +ejective consonant
    feature +syllabic consonant

    feature +aspirated consonant!glottal
    feature +labialized consonant!bilabial
    feature +velarized consonant!velar
    feature +palatalized consonant!palatal
    feature +pharyngealized consonant!pharyngeal

    feature +nasal-release plosive

    feature +round vowel
    feature closeness(close, close-mid, mid-close, open-mid, open) vowel
    feature backness(front, mid-back, back) vowel
    feature +rhoticized vowel

    feature +long all
    feature +nasal all
    feature +breathy all
end

symbols
    symbol i [close front -round]
    symbol y [close front +round]
    symbol ɨ [close mid-back -round]
    symbol ʉ [close mid-back +round]
    symbol ɯ [close back -round]
    symbol u [close back +round]

    symbol e [close-mid front -round]
    symbol ø [close-mid front +round]
    symbol ɘ [close-mid mid-back -round]
    symbol ɵ [close-mid mid-back +round]
    symbol ɤ [close-mid back -round]
    symbol o [close-mid back +round]

    symbol ə [mid-close mid-back]

    symbol ɛ [open-mid front -round]
    symbol œ [open-mid front +round]
    symbol ɜ [open-mid mid-back -round]
    symbol ɞ [open-mid mid-back +round]
    symbol ʌ [open-mid back -round]
    symbol ɔ [open-mid back +round]

    symbol a [open front -round]
    symbol ɶ [open front +round]
    symbol ɑ [open back -round]
    symbol ɒ [open back +round]


    symbol p [-voice bilabial plosive]
    symbol b [+voice bilabial plosive]
    symbol t [-voice alveolar plosive]
    symbol d [+voice alveolar plosive]
    symbol ʈ [-voice retroflex plosive]
    symbol ɖ [+voice retroflex plosive]
    symbol c [-voice palatal plosive]
    symbol ɟ [+voice palatal plosive]
    symbol k [-voice velar plosive]
    symbol g [+voice velar plosive]
    symbol q [-voice uvular plosive]
    symbol ɢ [+voice uvular plosive]
    symbol ʔ [-voice glottal plosive]

    symbol ɸ [-voice bilabial fricative]
    symbol β [+voice bilabial fricative]
    symbol f [-voice labiodental fricative]
    symbol v [+voice labiodental fricative]
    symbol θ [-voice dental fricative]
    symbol ð [+voice dental fricative]
    symbol s [-voice alveolar fricative]
    symbol ɬ [-voice alveolar fricative +lateral]
    symbol z [+voice alveolar fricative]
    symbol ɮ [+voice alveolar fricative +lateral]
    symbol ʃ [-voice postalveolar fricative]
    symbol ʒ [+voice postalveolar fricative]
    symbol ʂ [-voice retroflex fricative]
    symbol ʐ [+voice retroflex fricative]
    symbol ç [-voice palatal fricative]
    symbol ʝ [+voice palatal fricative]
    symbol x [-voice velar fricative]
    symbol ɣ [+voice velar fricative]
    symbol χ [-voice uvular fricative]
    symbol ʁ [+voice uvular fricative]
    symbol ħ [-voice pharyngeal fricative]
    symbol ʕ [+voice pharyngeal fricative]
    symbol h [-voice glottal fricative]
    symbol ɦ [+voice glottal fricative]

    symbol m [+voice bilabial nasal]
    symbol ɱ [+voice labiodental nasal]
    symbol n [+voice alveolar nasal]
    symbol ɳ [+voice retroflex nasal]
    symbol ɲ [+voice palatal nasal]
    symbol ŋ [+voice velar nasal]
    symbol ɴ [+voice uvular nasal]

    symbol ʙ [+voice bilabial trill]
    symbol r [+voice alveolar trill]
    symbol ʀ [+voice uvular trill]

    symbol ⱱ [+voice labiodental tap]
    symbol ɾ [+voice alveolar tap]
    symbol ɽ [+voice retroflex tap]

    symbol ʋ [+voice labiodental approximant]
    symbol ɹ [+voice alveolar approximant]
    symbol l [+voice alveolar approximant +lateral]
    symbol ɻ [+voice retroflex approximant]
    symbol ɭ [+voice retroflex approximant +lateral]
    symbol j [+voice palatal approximant]
    symbol ʎ [+voice palatal approximant +lateral]
    symbol ɰ [+voice velar approximant]
    symbol ʟ [+voice velar approximant +lateral]
end

diacritics
    diacritic ː [-long] => [+long]
    diacritic ̥◌ [+voice] => [-voice]
    diacritic ʰ [-aspirated] => [+aspirated]
    diacritic ʷ [-labialized] => [+labialized]
    diacritic ˠ [-velarized] => [+velarized]
    diacritic ʲ [-palatalized] => [+palatalized]
    diacritic ˤ [-pharyngealized] => [+pharyngealized]
    diacritic ⁿ [-nasal-release] => [+nasal-release]
end

rules
    rule fricatives-voice
        [fricative] => [+voice]
        ɦ => h
    end

    rule y-to-i
        y => i
    end

    rule rounded-to-u
        [+round] => u
    end

    rule b-bf
        b => >[+voice bilabial fricative]
    end

    rule pf-cluster
        [plosive] [fricative] => [+voice] >[mid-close mid-back]
    end

    rule schwa-loss
        [mid-close mid-back] => *
    end

    rule h-loss-lengthening
        [] h => [+long] *
    end

    rule voice-intervocalic
        [-voice] => [+voice] / [vowel] _ [vowel]
    end

    rule devoice-first
        [+voice] => [-voice] / $ _
    end

    rule word-final-vowel
        * => a / [consonant]<2:2> _ $
    end

    rule devoice-fricative-except-alveolar
        [fricative !alveolar] => [-voice]
    end

    rule devoice-word-final
        [+voice] => [-voice] / _ $
    end

    rule nasal-metathesis
        [plosive]$0 [nasal]$1 => []$1 []$0
    end

    rule nasal-assimilation
        [nasal] []$0(place) => []$0 []
    end

    rule bilabial-to-labiodental
        {ɸ β} => {f v}
    end

    rule palatalization
        {k g n} i => {c ɟ ɲ} * / _ [vowel]
    end
end

#{}
#()