module Main exposing (main)

import Browser exposing (Document, document)
import Html exposing (..)
import Html.Attributes exposing (class, id, value, src)
import Html.Events exposing (onClick)
import Http exposing (expectJson)
import Json.Decode as Decode
import File exposing (File)

type alias Flags = ()

type alias Metrics = { time: String, path: String }

type alias MetricsInfo = { error: String, time: String, path: String }

type Tab = Info | Upload

type InfoTab = Waiting | InfoDone (Result String Metrics)

type UploadTab = NotStarted | FileSelected File | InProgress () | UploadDone (Result String ())

type alias Model = { tab: Tab, info: InfoTab, upload: UploadTab }

type Message = RecievedInfo (Result Http.Error MetricsInfo)
    | MainButtonClick

-- utility --

make_metrics_request : Cmd Message
make_metrics_request = Http.get
    {
        url = "/api/metrics",
        expect = expectJson RecievedInfo decode_json_metrics
    }

decode_json_metrics : Decode.Decoder MetricsInfo
decode_json_metrics =
    Decode.map3 MetricsInfo
        (Decode.field "error" Decode.string)
        (Decode.field "time" Decode.string)
        (Decode.field "path" Decode.string)

-- init --

init : Flags -> (Model, Cmd Message)
init _ = (get_default_model, make_metrics_request)

get_default_model : Model
get_default_model =
    {
        tab = Info,
        info = Waiting,
        upload = NotStarted
    }

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
    case model.tab of
        Info -> get_info_model_id model.info
        Upload -> get_upload_model_id model.upload

get_model_main : Model -> List (Html Message)
get_model_main model =
    case model.tab of
        Info -> get_info_main model.info
        Upload -> get_upload_main model.upload

get_info_model_id : InfoTab -> String
get_info_model_id info =
    case info of
        Waiting -> "loading"
        InfoDone (Err _) -> "error"
        InfoDone (Ok _) -> "info"

get_upload_model_id : UploadTab -> String
get_upload_model_id _ = "loading"

get_info_main : InfoTab -> List (Html Message)
get_info_main info =
    case info of
        Waiting -> view_loading "Waiting for metrics from remote server..."
        InfoDone (Ok metrics) -> view_info metrics
        InfoDone (Err error) -> view_error error

get_upload_main : UploadTab -> List (Html Message)
get_upload_main _ =
    view_loading "This page hasn't been written yet..."

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
        view_info_pane "Timing"
            "The time taken from power-on until the demo-app is booted into." metrics.time,
        view_info_pane "Boot path"
            "The path taken by loadstone when deciding how to boot." metrics.path
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
update message model = (model, Cmd.none)
    |> update_info message
    |> update_upload message

update_info : Message -> (Model, Cmd Message) -> (Model, Cmd Message)
update_info message (model, command) =
    case message of
        RecievedInfo info -> ({ model | info = handle_recieved_info info }, command)
        MainButtonClick -> ({ model | info = Waiting }, make_metrics_request)

update_upload : Message -> (Model, Cmd Message) -> (Model, Cmd Message)
update_upload _ model_command = model_command

handle_recieved_info : Result Http.Error MetricsInfo -> InfoTab
handle_recieved_info result =
    case result of
        Ok info ->
            if info.error == "none" then
                InfoDone (Ok { time = info.time, path = info.path })
            else
                InfoDone (Err (get_info_error_message info.error))
        Err error ->
            InfoDone (Err (get_http_error_message error))

get_info_error_message : String -> String
get_info_error_message error =
    case error of
       "internal" -> "The server encountered an internal error."
       "device" -> "The server failed to initialise a connection to the device."
       "io" -> "The server initialised a connection to the device, but failed to communicate."
       "metrics" -> "The server recieved malformed metrics information from the device."
       _ -> "An unknown error occured when recieving the metrics."

get_http_error_message : Http.Error -> String
get_http_error_message error = 
    case error of
        Http.Timeout -> "The network timed out."
        Http.NetworkError -> "An unknown network error occured."
        Http.BadStatus status -> "HTTP error " ++ (String.fromInt status) ++ " occured."
        _ -> "An internal error occured."

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
