-- strings use single quotes
BEGIN;

DO $body$
    DECLARE
        -- use variables in order to be able to access properties using the dot operator
    BEGIN
        INSERT INTO public.events (
            id
            ,pubkey
            ,created_at
            ,kind
            ,tags
            ,content
            ,sig
        ) VALUES
        (
            '\xb042eae42505d83996af3694f47224128596c89a3ea1a7fd27ea43c8e559cf20'::bytea
            ,'\x7ecee90e906e56d7b20b2e76cdb83b786352d2bea53495e34ad556a989f7d39b'::bytea
            ,to_timestamp(1692815146)
            ,1
            ,$$[]$$::JSONB
            ,$$the internet would be a better place if it was shut down on tuesdays or the like$$
            ,'\x514ee6c22e9fb7e96d87e769ba024e9659a39ab313ed0d424b206e4bd21a5cf1ea9c4991420507c8242b5fe01a7011a49803c2ef19f748a253119e471d572b54'::bytea
        )
        ,(
            '\x1bb1d6acee88cd925c62e547e10c24ae65effe9286e4f1840e222643db76c833'::bytea
            ,'\xa6dff3503ca65ecf97371f2ba3348c2385e01c0212d1317dcb3a6d843ff08949'::bytea
            ,to_timestamp(1692815146)
            ,0
            ,$$[["p","f72657e01156d2c9b251111e73d58236dfb7de5ca69e1b53f0a938528f16c265"]]$$::JSONB
            ,$${"about":"weaponized stink eye","name":"bridget","picture":"https://coro.na/virus.png"}$$
            ,'\x16cc57d8e9d57690085b852d2ddab3248b63acf148625362cd7c1493682c89bf5b6d4bb9336f9f7a64a4a0d48ccb4084bccd858b7a1ab760a2cfc8700bc69150'::bytea
        )
        ,(
            '\x3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f'::bytea
            ,'\xbd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e'::bytea
            ,to_timestamp(1692815300)
            ,1
            ,$$[]$$::JSONB
            ,$$I have information that'll lead to the arrest of Kermit The Frog$$
            ,'\xce10841572b638aaebc97be44b28f1754265c8e48dd955fd131f54e26a32a6ebe4ab5c0343f8eecd7130cd0b736d2a3bd2ee44be03fdbf3206f67599a7d054d9'::bytea
        )
        ,(
            '\x6b4c4c5818219aca0055f38c1dc255907f5fbcf21b0332857cfddf697ac91cd7'::bytea
            ,'\xbd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e'::bytea
            ,to_timestamp(1692815400)
            ,1
            ,$$[["e","3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f"]]$$::JSONB
            ,$$I'm glad people are paying attention. Information will be released soonTM. Meanwhile, I'll be selling Henson-gate tank-tops and jerseys. Links in my bio$$
            ,'\x5bfca647096acaa0e311b0efd7ca3a3c602be5cbe858259d32fe26ab6dbc80366b94e29ff7f4e5ed694834243b27e04644df1efd6384abe7eefffd1ab5a89a9e'::bytea
        )
        ,(
            '\xe974080cde211594bbf3197ec9bceb43a27ed67366671fa69d5b65c1848d2f6e'::bytea
            ,'\x167c3b7d2640757b2d276c0f9c50d6820aa45208f97acd06a76920e532639c20'::bytea
            ,to_timestamp(1692815500)
            ,1
            ,$$[["e","3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f"]]$$::JSONB
            ,$$Henson-gate$$
            ,'\x1cd8cfe534f6904bee6742eed035954e8bd15d8ab31e5c4712810c4ec954ebfb7be571656b8421d37c7b7e18daae14df6c82d727821f69e9df9c2241e3624d6c'::bytea
        )
;
    END;
$body$ LANGUAGE PLpgSQL;

-- you can bypass the DO section though
-- INSERT UPDATE STUFF
COMMIT;
-- ROLLBACK;
