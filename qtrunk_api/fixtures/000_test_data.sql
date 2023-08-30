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
            ,'\xff8925580d86f8cbf0de60eca4e1984e526bbf273801dde7824e2c1ee23e6ab70f41f929092c32440e40cfd45f4532dfc03be28b5fc271fd51825b7aafdd0104'::bytea
        )
        ,(
            '\x1bb1d6acee88cd925c62e547e10c24ae65effe9286e4f1840e222643db76c833'::bytea
            ,'\xa6dff3503ca65ecf97371f2ba3348c2385e01c0212d1317dcb3a6d843ff08949'::bytea
            ,to_timestamp(1692815146)
            ,0
            ,$$[["p","f72657e01156d2c9b251111e73d58236dfb7de5ca69e1b53f0a938528f16c265"]]$$::JSONB
            ,$${"about":"weaponized stink eye","name":"bridget","picture":"https://coro.na/virus.png"}$$
            ,'\x3fd86ad14a171043b1ca9cacb58377bf8091288394f80e20ee30ed4e9adac7564045d2596eb3b95b6050ef8f2dd0df4a5b702f07d3ade44082934cce4fe869bb'::bytea
        )
        ,(
            '\x3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f'::bytea
            ,'\xbd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e'::bytea
            ,to_timestamp(1692815300)
            ,1
            ,$$[]$$::JSONB
            ,$$I have information that'll lead to the arrest of Kermit The Frog$$
            ,'\xf7d63de3be8c33334363098e53507cd5dc211d73e9e57be254c5b1035718cc4678501bc1a18d1afabe9b9b35968ba41440cae5bf46db3ebeb3ab1cf1eb359fd9'::bytea
        )
        ,(
            '\x6b4c4c5818219aca0055f38c1dc255907f5fbcf21b0332857cfddf697ac91cd7'::bytea
            ,'\xbd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e'::bytea
            ,to_timestamp(1692815400)
            ,1
            ,$$[["e","3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f"]]$$::JSONB
            ,$$I'm glad people are paying attention. Information will be released soonTM. Meanwhile, I'll be selling Henson-gate tank-tops and jerseys. Links in my bio$$
            ,'\x8a245231616d31b5ff13401884e798bf6947570c29093ac3e3850a72991c02e6deed77f60ca9921f4b4901648ed03a3e70fe5d59bc623d6c9afb824e270e60f5'::bytea
        )
        ,(
            '\xe974080cde211594bbf3197ec9bceb43a27ed67366671fa69d5b65c1848d2f6e'::bytea
            ,'\x167c3b7d2640757b2d276c0f9c50d6820aa45208f97acd06a76920e532639c20'::bytea
            ,to_timestamp(1692815500)
            ,1
            ,$$[["e","3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f"]]$$::JSONB
            ,$$Henson-gate$$
            ,'\xf7a4e72838ca5062aadfb9d56bc837012374bdc4050f7aa004f3c81881e0d303a1ca6b45aa6f72555e7bda0f571d5572a8acd9b284ed5809d5f1ff652f06b0f3'::bytea
        )
;
    END;
$body$ LANGUAGE PLpgSQL;

-- you can bypass the DO section though
-- INSERT UPDATE STUFF
COMMIT;
-- ROLLBACK;
