feature_def
    switch value(A, B, C, D, E, F) root

    feature +mess B

    feature +col11 A
    feature +col12 B
    feature +col13 C
    feature +col14 (D, E)

    feature +Aex (B, C, D, E, F)
    feature +Bex (A, C, D, E, F)
    feature +Cex (A, B, D, E, F)
    feature +Dex (A, B, C, E, F)
    feature +Eex (A, B, C, D, F)
    feature +Fex (A, B, C, D, E)

    feature +col21 A
    feature +col22 B
    feature +col23 C
    feature +col24 (D, E)
end