module Main exposing (main)

import Browser exposing (Document, document)
import Html exposing (..)
import Html.Attributes exposing (class, id, value, src)
import Html.Events exposing (onClick)
import Browser.Navigation
import Http exposing (expectJson)
import Json.Decode as Decode

type alias Flags = ()

type alias Metrics = { time: String, path: String }

type alias MetricsInfo = { error: String, time: String, path: String }

type Model = WaitingForInfo
    | Error String
    | Info Metrics

type Message = RecievedInfo (Result Http.Error MetricsInfo)
    | MainButtonClick

-- init --

init : Flags -> (Model, Cmd Message)
init _ =
    (
        WaitingForInfo,
        Http.get {
            url = "/api/metrics",
            expect = expectJson RecievedInfo decode_json_metrics
        }
    )

decode_json_metrics : Decode.Decoder MetricsInfo
decode_json_metrics =
    Decode.map3 MetricsInfo
        (Decode.field "error" Decode.string)
        (Decode.field "time" Decode.string)
        (Decode.field "path" Decode.string)

-- view --

view : Model -> Document Message
view model =
    {
        title = view_title,
        body = view_body model
    }

view_title : String
view_title = "Loadstone Metrics"

view_body : Model -> List (Html Message)
view_body model =
    [
        header [] [
            h1 [] [ text view_title ]
        ],
        main_ [ id (get_model_id model) ] (
            get_model_main model
        )
    ]

get_model_id : Model -> String
get_model_id model =
    case model of
        WaitingForInfo -> "loading"
        Info _ -> "info"
        Error _ -> "error"

get_model_main : Model -> List (Html Message)
get_model_main model =
    case model of
        WaitingForInfo -> view_loading "Waiting for metrics from remote server..."
        Info metrics -> view_info metrics
        Error message -> view_error message

view_loading : String -> List (Html Message)
view_loading message =
    [
        div [ id "container" ] [
            img [ src "/loading.gif" ] [],
            h2 [] [ text message ]
        ]
    ]

view_info : Metrics -> List (Html Message)
view_info metrics =
    [
        view_info_pane "Timing" "The time taken from power-on until the demo-app is booted into." metrics.time,
        view_info_pane "Boot path" "The path taken by loadstone when deciding how to boot." metrics.path
    ]

view_error : String -> List (Html Message)
view_error message =
    [
        div [ id "container" ] [
            h2 [] [ text "Failed to retrieve metrics" ],
            p [] [ text message ],
            button [ onClick MainButtonClick ] [ text "Reload" ]
        ]
    ]

view_info_pane : String -> String -> String -> Html Message
view_info_pane name description value =
    div [ class "info-pane"] [
        div [ class "info-pane-name" ] [
            h2 [] [ text name ],
            span [] [ text description ]
        ],
        div [ class "info-pane-value" ] [ text value ]
    ]

-- update --

update : Message -> Model -> (Model, Cmd Message)
update message model =
    case model of
        WaitingForInfo -> update_waiting_for_info message
        Info _ -> (model, Cmd.none)
        Error m -> update_error m message

update_waiting_for_info : Message -> (Model, Cmd Message)
update_waiting_for_info message =
    case message of
        RecievedInfo (Err error) -> (Error (get_http_error_message error), Cmd.none)
        RecievedInfo (Ok info) -> (handle_recieved_info info, Cmd.none)
        _ -> (WaitingForInfo, Cmd.none)

update_error : String -> Message -> (Model, Cmd Message)
update_error m message =
    case message of
       MainButtonClick -> (Error m, Browser.Navigation.reload)
       _ -> (Error m, Cmd.none)

get_http_error_message : Http.Error -> String
get_http_error_message error = 
    case error of
        Http.Timeout -> "The network timed out."
        Http.NetworkError -> "An unknown network error occured."
        Http.BadStatus status -> "Error " ++ (String.fromInt status) ++ " occured."
        _ -> "An internal error occured."

handle_recieved_info : MetricsInfo -> Model
handle_recieved_info info =
    if info.error == "none" then
        Info { time = info.time, path = info.path }
    else
        Error (get_info_error_message info.error)

get_info_error_message : String -> String
get_info_error_message error =
    case error of
       "device" -> "The server failed to initialise a connection to the device."
       "io" -> "The server initialised a connection to the device, but failed to communicate."
       "metrics" -> "The server recieved malformed metrics information from the device."
       _ -> "An unknown error occured when recieving the metrics."

-- subscriptions --

subscriptions : Model -> Sub Message
subscriptions _ = Sub.none

-- main --

main : Program Flags Model Message
main =
    document {
        init = init,
        view = view,
        update = update,
        subscriptions = subscriptions
    }
