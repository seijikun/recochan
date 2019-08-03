# recochan
Reco-Chan is a recommendation engine based on the FunkSVD.
This is the approach that Simon Funk used in the Netflix challenge 2006.
This approach tries to extract the abstract notion of "*features*" from the given ratings. A feature could for example be *genre Action*.
Each of those features is mapped to the rated items, as well as the users that rated them.
Internally, the model works with **n** feature values per item, as well as per user.
If the value of one feature for a given item is high, the item "contains" the feature. For example an anime is in the action genre.
If the value of one feature for a given user is high, the user "likes" that feature. For example: The user likes action animes.

Note, however, that the algorithm doesn't know about the concept of Genres or anything item-related. It extracts these relations by chance - which also means that it will not possible to specifiy what each feature exactly means.
The more features Reco-Chan trains, the more concepts are supported. But when there are less actual features than Reco-Chan is trying to find, the engine will start to model the noise - which will result in bad predictions.

As the name **Reco-Chan** may suggests, its original intention was to make recommendations for animes.
For this reason, **Reco-Chan** has a [tsundere](https://en.wikipedia.org/wiki/Tsundere) personality, that it displays in the log messages.

# Recommendations
Reco-Chan can produce the following:
- Personal item recommendations for a given user ("You will probably want to watch **x** next..");
- Find similar users to a given user (users that probably have the same taste)
- Find similar animes to a given anime (animes that may be in the same genre, with the same setting)

# Configuring
Reco-Chan is configurable.
For this, Reco-Chan tries to find the file `recochan.json` within the current working folder (i.e. the folder that Reco-Chan is started from).
When she does not find the file, she complains about the file missing and won't start.

Reco-Chan does have a couple of default values for some configuration parameters. If you want to use the default value - you can simply not mention the configuration option in question in `recochan.json` at all. Internally, Reco-Chan merges the options provided within the file, with the internal default values.

To be able to use Reco-Chan, you need to configure the dataprovider you want to use. (see below)
Example configuration file:
```json
{
	"api": {
		"bind": "0.0.0.0",
		"port": 1337
	},
	"dataprovider": {
		"type": "SQL",
		"connection_string": "mysql://root:password@localhost:3307/database_name",
		"where_clause": "",
		"aid_name": "animeid",
		"uid_name": "userid",
		"rating_name": "rating",
		"table_name": "ratings"
	},
	"retrain_every_sec": 86400
}
```

## Overview
The following table tries to give an overview of all of the currently supported configuration values that Reco-Chan understands.
It tries to display the hierarchy within the configuration file, and provides a small explanation for each option, as well as the current default value (if any).
If you are fine with any of the given defaults, simply don't mention the option in the configuration file.

| Section           | Effect                                                                                           | Default   |
|-------------------|--------------------------------------------------------------------------------------------------|-----------|
| **api**           | Section that contains any configuration regarding Reco-Chan's API.                               |     -     |
| .bind             | IP-Address that the webserver will bind to, to provide the API                                   | 127.0.0.1 |
| .port             | Port that the webserver will bind to.                                                            | 1337      |
| **dataprovider**  | This will contain the configuration for the dataprovider that should be used.                    |     -     |
| ...               | (Have a look at the dataprovider section below)                                                  |     -     |
| retrain_every_sec | Interval (in seconds) in which Reco-Chan should automatically retrain the used prediction model. | 86400     |

# Dataproviders
ReckoChan has a generic interface called `RatingDataProvider`. At the moment, Reco comes with two implementations for this trait:
**Notice:** Only one dataprovider can be used at a time. The configuration file is meant to select one of the supported ones before starting Reco-Chan!

## SQLDataProvider
SQLDataProvider is a configurable `RatingDataProvider` implementation that gets the user ratings from a SQL database.
You can configure:
- Name of the `animeid` column
- Name of the `userid` column
- Name of the `rating` column
- Name of the `table`
- An (optional) `WHERE` clause, that will filter ratings before importing them (`WHERE` is automatically prepended, when the option string is not empty)

#### Example configuration:
```json
[...]
	"dataprovider": {
		"type": "SQL",
		"connection_string": "mysql://root:password@localhost:3307/database_name",
		"where_clause": "",
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