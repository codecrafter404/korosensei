pub fn get_uninitalized_timestamp() -> DateTime<Utc> {
    return NaiveDate::from_ymd_opt(1, 1, 1)
        .unwrap()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_local_timezone(Utc)
        .unwrap();
}
