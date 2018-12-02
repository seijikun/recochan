# recochan
Reco-Chan is a recommendation engine based on the FunkSVD.
This is the approach that Simon Funk used in the Netflix challenge 2006.
As the name may suggest, it's original intention was to make recommendations for animes.

# Recommendations
At the moment, Reco-Chan can produce personal recommendations. In other words: You tell Reco-Chan what user you would like to have anime recommendations for, and Reco-Chan will answer with a list of all animes and corresponding predicted rating (sorted descending by rating).

# Configuring
Reco-Chan is configurable. To be able to use it, you need to configure the dataprovider you want to use. (see below)
You can also configure the Web-API that Reco-Chan exposes:
```json
{
    "api": {
        "bind": "127.0.0.1",
        "port": 1337
    }
}
```

# Dataproviders
ReckoChan has a generic interface called `RatingDataProvider`. At the moment, Reco comes with two implementations for this trait:

## SQLDataProvider
SQLDataProvider is a configurable `RatingDataProvider` implementation that gets the user ratings from a SQL database.
You can configure:
- Name of the `animeid` column
- Name of the `userid` column
- Name of the `rating` column
- Name of the `table`

#### Example configuration:
```json
{
	"dataprovider": {
		"type": "SQL",
		"connection_string": "mysql://root:password@localhost:3307/mysql",
		"aid_name": "animeid",
		"uid_name": "userid",
		"rating_name": "rating",
		"table_name": "ratings"
	}
}
```

## TestDataCsvProvider
This is a `RatingDataProvider` that is mainly meant for test-purposes. It reads all csv files within the configurable path.
The filename should be: `<animeid>.csv`.
The content of such a file should be of the format:
```csv
<userid> <rating>
```
**Attention:** rating is an integer between 0 and 10, the dataprovider halfes this value.

#### Example configuration:
```json
{
	"dataprovider": {
		"type": "TestCSV",
		"path": "/tmp/recommendations/"
	}
}
```