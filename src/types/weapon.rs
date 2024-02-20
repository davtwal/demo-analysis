

pub enum WeaponSlot {
    Primary = 0,
    Secondary = 1,
    Melee = 2,
    Pda = 3,
    Pda2 = 4
}

/*
THE WEAPON TYPE TREE:
    CWeaponBase - a bunch of info
        CWeaponBaseGun
            CTFGrenadeLauncher - m_flDetonateTime, m_iCurrentTube, m_iGoalTube
                CTFCannon
            CTFSMG
                CTFChargedSMG - m_flMinicritCharge
            CTFJar
                CTFCleaver
                CTFJarGas
                CTFJarMilk
                CTFThrowable - base for spellbook; m_flChargeBeginTime
                    CTFSpellBook - spell stuff ig
            CTFPipebombLauncher - under PipebombLauncherLocalData: m_iPipebombCount, m_flChargeBeginTime
                CTFCompoundBow - m_bArrowAlight, m_bNoFire
            CTFRocketLauncher
                CTFCrossbow - m_flRegenerateDuration, m_flLastUsedTimestamp
                CTFRaygun - m_bUseNewProjectileCode

*/

pub enum WeaponType {
    // Scout
    Scattergun,     // CTFScatterGun
    BabyFaces,      // CTFPEPBrawlerBlaster


    ScoutPistol,    // CTFPistol_Scout - Stock, Lugermorph, CAPPER
    ScoutPistol2,   // CTFPistol_ScoutSecondary - Winger & Pretty-boys

    Bat,            // CTFBat - Stock + reskins, basher, sun-stick, atomizer
    Fish,           // CTFBat_Fish - Fish & unarmed combat
    Sandman,        // CTFBat_Wood - Sandman
    WrapAssassin,   // CTFBat_Giftwrap

    RocketLauncher,
    CowMangler,

    GrenadeLauncher,// CTFGrenadeLauncher
    StickyLauncher, // CTFPipebombLauncher


    Crossbow,
    Medigun,
}

pub struct Medigun {
    pub(crate) entity: u32,
    pub owner: u32,
    pub heal_target: u32,
    pub is_healing: bool,
    pub is_holstered: bool,
}

