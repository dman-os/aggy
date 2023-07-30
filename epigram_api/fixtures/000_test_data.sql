-- strings use single quotes
BEGIN;

DO $body$
    DECLARE
        -- use variables in order to be able to access properties using the dot operator
    BEGIN
        INSERT INTO grams.grams (
            id
            ,content
            ,mime
            ,parent_id
            ,sig
            ,author_pubkey
            ,author_alias
            ,author_notif_email
        ) 
        VALUES 
        (
            '\x4776309475fa7460e046126bd8d9453a88cb4548bd35e5f7cfafbf9a0ecb64d5'::bytea
            ,$$I wan't you to know, I wan't you to know that I'm awake.$$
            ,'text/html'
            ,NULL
            ,'\x4574a3daaffb3a29704ee1c9937217a4a72f96afc8c4f6e9de5ea7bab85b0b6fa05c692efb380ea92e9fc6058cd683b105bc9245222cb8bef2572dbad6075d09'::bytea
            ,'\xc993d470f9e138ad4c94bb897e2733f6314e38e9dcb58e661d224234b55c7b98'::bytea
            ,'use1'
            ,'use1@aggy.news'
        )
        ,(
            '\xe20b19235696d1469fd00f44a73de9111f0983227682284fa7526b66c19cded7'::bytea
            ,$$And I hope you're asleep.$$
            ,'text/html'
            ,'\x4776309475fa7460e046126bd8d9453a88cb4548bd35e5f7cfafbf9a0ecb64d5'::bytea
            ,'\xddf5cf0fc11586706931a2ed25cd5dace45db6a9257fbc8f242cd0e98433c6854961107c01fc788b7db5ce0f97944b333e874a643cc130b6723dc29779571f0f'::bytea
            ,'\x108a880634a69715e6d5ccb79888530fe2a204037e5d917d9f750576a084d1a3'::bytea
            ,'fideroth'
            ,'fideroth@aggy.news'
        )

        ,(
            '\x2d7ebd96468b0e889887251864c337a8ee042200cd92d93fa77ec6de44049fc0'::bytea
            ,$$*air guitars madly*$$
            ,'text/html'
            ,'\xe20b19235696d1469fd00f44a73de9111f0983227682284fa7526b66c19cded7'::bytea
            ,'\x42929747078a1e4e5301a7dc8b1092ee8fa091770282d9ddfe3d55051e5a9cee32d2d8a94f4980ccbacee36044fb4be4875def17ae5386653423874e8a9ea208'::bytea
            ,'\xc993d470f9e138ad4c94bb897e2733f6314e38e9dcb58e661d224234b55c7b98'::bytea
            ,'use1'
            ,'use1@aggy.news'
        )
        ,(
            '\xcc212c4940c8e5872f67493f14c7817bf0e144d34b13d798a116cd606b58c616'::bytea
            ,$$*sads doggly*$$
            ,'text/html'
            ,'\x2d7ebd96468b0e889887251864c337a8ee042200cd92d93fa77ec6de44049fc0'::bytea
            ,'\xc7378d3ee55f7aab0cd284d3a718a62ab20829d08ea0589a12cbfcf6321b5b010bc3f761fbe33ac420cf7a4b69b6dfb39aa64fddf27c5abf273873f007a4b80a'::bytea
            ,'\x108a880634a69715e6d5ccb79888530fe2a204037e5d917d9f750576a084d1a3'::bytea
            ,'fideroth'
            ,'fideroth@aggy.news'
        )
        ,(
            '\x8abe2cbf5c5d44bc4a507358e18587d60c99b26b563344a8f1decde6d1b218a7'::bytea
            ,$$What gives?$$
            ,'text/html'
            ,'\x4776309475fa7460e046126bd8d9453a88cb4548bd35e5f7cfafbf9a0ecb64d5'::bytea
            ,'\x73e41fa332606f525e139ae93ed469bf4e5c3618dbe08de6f1612c98c1d770e0c142f32a5db478bf8d4fe3edab1b2a3df724c28589509788aea96028f140660c'::bytea
            ,'\x4c5d8ad048ea438fdfb4b96054e955877e9c78336e09572479817fb84d547345'::bytea
            ,'the_i18n_man'
            ,'the_i18n_man@aggy.news'
        )
        ,(
            '\xd8a1909a74a7aec804bcf2adf4ac5057e8f5e1a7664d188988775f64e3464a1f'::bytea
            ,$$What doesn't?$$
            ,'text/html'
            ,'\x8abe2cbf5c5d44bc4a507358e18587d60c99b26b563344a8f1decde6d1b218a7'::bytea
            ,'\x274b9809ecda35824fbb1d42a2a472f60e3e7c0c2956612158b94ad8d57c261cbd5badc0710b94a6629ab0b467a81ec10d28ad209d2fc48c7f486fb4175b0700'::bytea
            ,'\x53b38db29fb2136d0100486db1c485652f6db05c095ca3b60fbcc2deaf549f15'::bytea
            ,'wgt'
            ,'wgt@aggy.news'
        )
        ,(
            '\x30d967a63dc2f8ff6aaa9478b427ea32cd91e2f885f09680df08be99d0ab6437'::bytea
            ,$$Stop redditing!!!$$
            ,'text/html'
            ,'\x4776309475fa7460e046126bd8d9453a88cb4548bd35e5f7cfafbf9a0ecb64d5'::bytea
            ,'\x1bf3c4d51008a6c9a7c2744a2764961e0aa85a3d6e086eff2254a7487a98ff7780037a01cc250f9632f1824b68f9a975ba60e0e4eb05fad86aaa23e1706e420f'::bytea
            ,'\x46afd78724c1a0c1f4d6990646153a2fbfaf1801c519d9ea3ad0a2d037b8214d'::bytea
            ,'ftw'
            ,'ftw@aggy.news'
        )
        ;
    END;
$body$ LANGUAGE PLpgSQL;

-- you can bypass the DO section though
-- INSERT UPDATE STUFF
COMMIT;
-- ROLLBACK;
