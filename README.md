# recochan
Reco-Chan is a recommendation engine based on the FunkSVD.
This is the approach that Simon Funk used in the Netflix challenge 2006.
As the name may suggest, it's original intention was to make recommendations for animes.

# Recommendations
At the moment, Reco-Chan can produce personal recommendations. In other words: You tell Reco-Chan what user you would like to have anime recommendations for, and Reco-Chan will answer with a list of all animes and corresponding predicted rating (sorted descending by rating).

# Configuring
Reco-Chan is configurable. To be able to use it, you need to configure the dataprovider you want to use. (see below)
Example configuration file:
```json
{
	"api": {
		"bind": "0.0.0.0",
		"port": 1337
	},
	"dataprovider": {
		"type": "SQL",
		"connection_string": "mysql://root:password@localhost:3307/mysql",
		"aid_name": "animeid",
		"uid_name": "userid",
		"rating_name": "rating",
		"table_name": "ratings"
	},
	"retrain_every_sec": 10
}
```

## Overview
| Section           | Effect                                                                                           |
|-------------------|--------------------------------------------------------------------------------------------------|
| **api**           | Section that contains any configuration regarding Reco-Chan's API.                               |
| .bind             | IP-Address that the webserver will bind to, to provide the API                                   |
| .port             | Port that the webserver will bind to.                                                            |
| **dataprovider**  |  This will contain the configuration for the dataprovider that should be used.                   |
| ...               | (Have a look at the dataprovider section below)                                                  |
| retrain_every_sec | Interval (in seconds) in which Reco-Chan should automatically retrain the used prediction model. |

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
[...]
	"dataprovider": {
		"type": "SQL",
		"connection_string": "mysql://root:password@localhost:3307/mysql",
		"aid_name": "animeid",
		"uid_name": "userid",
		"rating_name": "rating",
		"table_name": "ratings"
	}
[...]
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
[...]
	"dataprovider": {
		"type": "TestCSV",
		"path": "/tmp/recommendations/"
	}
[...]
```