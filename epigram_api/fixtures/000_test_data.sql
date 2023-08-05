-- strings use single quotes
BEGIN;

DO $body$
    DECLARE
        -- use variables in order to be able to access properties using the dot operator
    BEGIN
        INSERT INTO grams.grams (
            id
            ,content
            ,coty
            ,parent_id
            ,sig
            ,author_pubkey
            ,author_alias
            ,author_notif_email
        ) 
        VALUES 
        (
            '\x26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61'::bytea
            ,$$I wan't you to know, I wan't you to know that I'm awake.$$
            ,'text/html'
            ,NULL
            ,'\xcc048f2de1d7b3bf0608a3b89a1a71e4f8c8db4049980dca31efe48271ebaabb0572a62bd0346348f5ae09d0b1fd7a530ecab974fc6e474fac46b03127f19802'::bytea
            ,'\x691d917d665d04bb35b65ff896478b9dd59af81ade6c6d7a98d9c19666147c87'::bytea
            ,'use1'
            ,'use1@aggy.news'
        )
        ,(
            '\x863a254a782fae5bcde8629a01a5591a89d1e6bfc531ce5ae4443e149dc29d77'::bytea
            ,$$And I hope you're asleep.$$
            ,'text/html'
            ,'\x26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61'::bytea
            ,'\x6223912f4339bf83829467a32a67cb5e87988f710b65202f86fbb43fbf194f941895e2f5578f205254132ed1d7b1ae8ce712057f19eccccdeb4c20a871fb3e0e'::bytea
            ,'\xd093f5a4cbc24177a52b4c7b3050c2380f0da88162b84c30f8ff44bbe4e86c77'::bytea
            ,'fideroth'
            ,'fideroth@aggy.news'
        )
        ,(
            '\xa3bf486c93ed2e6d5d61ecff467670eee74c85942441ddd9422d1139b8044c5b'::bytea
            ,$$*air guitars madly*$$
            ,'text/html'
            ,'\x863a254a782fae5bcde8629a01a5591a89d1e6bfc531ce5ae4443e149dc29d77'::bytea
            ,'\x8f2f73e71d7fc723e4bf0ccafec7ab6726ac0d9c61c6d3d3f4d64419e1a1109fa1502afd8f578100b33e31221fc8cce19ee526f4b6da6424feb6ebd0ccc7be00'::bytea
            ,'\x691d917d665d04bb35b65ff896478b9dd59af81ade6c6d7a98d9c19666147c87'::bytea
            ,'use1'
            ,'use1@aggy.news'
        )
            ,(
            '\x8e007922fb38461df02aae6409276ba8f9eb39c64066c585ffccb0023146cd79'::bytea
            ,$$*sads doggly*$$
            ,'text/html'
            ,'\xa3bf486c93ed2e6d5d61ecff467670eee74c85942441ddd9422d1139b8044c5b'::bytea
            ,'\x24f497b3bd42f676538fe974cc7e233c74605880b033ec7964db1734eb1aea9d7c530ee9c41376e5cad4c530bf3bb34ef75f9a2a0044ec0d2dd838e1611b2f00'::bytea
            ,'\xd093f5a4cbc24177a52b4c7b3050c2380f0da88162b84c30f8ff44bbe4e86c77'::bytea
            ,'fideroth'
            ,'fideroth@aggy.news'
        )
            ,(
            '\xb9c347e6bdc757c3068d1f3a3c6b9d8e21af1c6608724bed0efb9d0e2e0ac1f7'::bytea
            ,$$What gives?$$
            ,'text/html'
            ,'\x26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61'::bytea
            ,'\x4a8521a7b30fb07b63c8d028efb2366aa0d2356449a08817a4328bda28381ed4c5b7527ab5c956817b437981f6320ea4dbca4a7ab25f73e1157417e294196c06'::bytea
            ,'\x5ee665a116a24fd6a6f6b60f44cd8424b67282258ade3e0d7f84abcf9cf94bed'::bytea
            ,'the_i18n_man'
            ,'the_i18n_man@aggy.news'
        )
            ,(
            '\x17d2d48476d24b39f6fe339027581bc51b4b52bc0abd36d2abbad8ef09a2656e'::bytea
            ,$$What doesn't?$$
            ,'text/html'
            ,'\xb9c347e6bdc757c3068d1f3a3c6b9d8e21af1c6608724bed0efb9d0e2e0ac1f7'::bytea
            ,'\xe6de2657ad68b530572f945bb6b674bc058f34eb20aa74f2ce80deb98cc914fbfb5de0af82f8e18a2b9bb90a55e26aa83629f863c7a9b6e740f9abe559a11b02'::bytea
            ,'\xbf9ba40d4bf80f00a5e44d52049bd4f97fc7aacca5c954adf91953b5c9a0c664'::bytea
            ,'wgt'
            ,'wgt@aggy.news'
        )
            ,(
            '\xc811bd14ca4bf8a318166e59117342cfed93adbde95249cc5d7ea84195b5201e'::bytea
            ,$$Stop redditing!!!$$
            ,'text/html'
            ,'\x26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61'::bytea
            ,'\xb4bcc052e2a10603f29d841299b8b39f363e68c0aaf0fb39e3f75daba71380dd088ad8f2a8f49c1d421e17c50754c43ab2260965aef0fd2b62498458f3e60306'::bytea
            ,'\x2e5d6e21a133a30ac7e7685e9a67ec3d70db810be1b8dc5af5a065b7f4d2a0aa'::bytea
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
