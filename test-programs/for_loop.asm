.ORIG	x3000
    AND r0, r0, 0
    LOOP
        ADD r0, r0, 1
        ADD r1, r0, -10
    BRn LOOP
.END
