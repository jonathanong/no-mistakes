-- A trailing token after the complete body makes the outer DO wrapper invalid.
DO 'BEGIN PERFORM 1; END' unexpected;
