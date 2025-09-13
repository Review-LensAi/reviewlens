package main

import "database/sql"

func vulnerable(db *sql.DB, user string) {
    query := "SELECT * FROM users WHERE name = '" + user + "'"
    db.Query(query)
}
