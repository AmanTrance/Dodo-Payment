# MOCK EVENT BASED TRANSACTION SYSTEM

## HOW TO SETUP THE SYSTEM (WITH DOCKER)

> Just do `docker compose up -d`


# REQUEST/RESPONSE DOCS

*NOTE* ALL Error Responses always has a field `error` with the error message
*NOTE* Minimal checks are there for email format and password length as it is a mock system only for demo

*** POST /v1/auth/signup ***


REQUEST application/json

```
{
    "username": "test",
    "email": "test@gmail.com",
    "password": "test"
}
```

RESPONSE application/json

```
{
    "token": "some token",
    "upi_id": "test@dodo"
}
```

*** POST /v1/auth/signin ***


REQUEST application/json 

```
{
    "email": "test@gmail.com",
    "password": "test"
}
```

RESPONSE application/json

```
{
    "token": "some token"
}
```

*** PUT /v1/profile/update ***


* Requires Authorization header with the provided JWT

REQUEST application/json 

```
{
    "city": "any",
    "state": "any",
    "country": "any",
    "avatar": "any"
}
```

RESPONSE application/json

```
{
    "message": "Success"
}
```

*** GET /v1/profile/get ***


* Requires Authorization header with the provided JWT

RESPONSE application/json

```
{
    "created_at": "2025-05-21T13:00:33.558487",
    "username": "test",
    "email": "test@gmail.com",
    "city": "any",
    "state": "any",
    "country": "any",
    "avatar": "any"
}
```

*** GET /v1/events ***

* Requires Authorization header with the provided JWT

RESPONSE test/event-stream


*** GET /v1/upi/list ***

* Requires Authorization header with the provided JWT

RESPONSE application/json

```
[
    {
        "upi_id": "test@dodo",
        "created_at": "some time",
        "is_default": true/false
    }
]
```

*** POST /v1/upi/fund ***

* Requires Authorization header with the provided JWT

REQUEST application/json 

```
{
    "upi_id": "test@dodo",
    "amount": 1000
}
```

RESPONSE application/json

```
{
    "message": "Success"
}
```

*** POST /v1/transaction/create ***

* Requires Authorization header with the provided JWT

REQUEST application/json 

```
{
    "to": "test@dodo",
    "amount": 1000
}
```

RESPONSE application/json

```
{
    "message": "will be processed shortly"
}
```

*** GET /v1/transaction/list ***

* Requires Authorization header with the provided JWT

RESPONSE application/json

```
[

]
```

*** GET /v1/account/balance ***

* Requires Authorization header with the provided JWT

RESPONSE application/json

```
{
    "balance": 1000
}
```