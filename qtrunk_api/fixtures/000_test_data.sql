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
            '\x8a8228471b5a0de4ec033749f90d2dbef1247b424bbb4c94b630575d84e782ce'::bytea
            ,'\xbd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e'::bytea
            ,to_timestamp(1692815146)
            ,1
            ,$$[]$$::JSONB
            ,$$I have information that'll lead to the arrest of Kermit The Frog$$
            ,'\xaa1b89e0f86dca2e930c57f2311dedcc30c9a2ff13f56dcfc0cf018c8f5e2d867bacd9889fafc5cd6e33acb8e8bc17de7655bb67f5813477cd0c9f0de0d5bfb8'::bytea
        )
        ,(
            '\xec41a05e3f5921d1b16b807f5c6e77b54349819fc59a998d341e8e15bda378e6'::bytea
            ,'\xbd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e'::bytea
            ,to_timestamp(1692815146)
            ,1
            ,$$[["e","8a8228471b5a0de4ec033749f90d2dbef1247b424bbb4c94b630575d84e782ce"]]$$::JSONB
            ,$$I'm glad people are paying attention. Information will be released soonTM. Meanwhile, I'll be selling Henson-gate tank-tops and jerseys. Links in my bio$$
            ,'\xfbeaf4ac101e8252e9a4cce13e8004b4232b5a9e1ec236e49312151f87b24025fea94c6f25171eb7e68b048fc9b44c0f5e443284f65ca9087f15af0cef6efcb3'::bytea
        )
        ,(
            '\x51acf76b8a5676950bd8b40bff62ee652b7d672cb95d029a466185eb1291dc5a'::bytea
            ,'\x167c3b7d2640757b2d276c0f9c50d6820aa45208f97acd06a76920e532639c20'::bytea
            ,to_timestamp(1692815146)
            ,1
            ,$$[["e","8a8228471b5a0de4ec033749f90d2dbef1247b424bbb4c94b630575d84e782ce"]]$$::JSONB
            ,$$Henson-gate$$
            ,'\x14ee5f9a5e2aa95064e297a4801b4b0c12392d601c67fd3a74b8dfebe2baa525b129eaa51d713531693ff18eaf365c14cb19a301c553629b9d7853950dc1bd55'::bytea
        )
;
    END;
$body$ LANGUAGE PLpgSQL;

-- you can bypass the DO section though
-- INSERT UPDATE STUFF
COMMIT;
-- ROLLBACK;
