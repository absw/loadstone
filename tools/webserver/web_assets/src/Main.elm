port module Main exposing (main)

import Browser exposing (Document, document)
import Html exposing (..)
import Html.Attributes exposing (class, id, value, src)
import Html.Events exposing (onClick, onInput)
import Browser.Navigation

port report_websocket_state : (String -> msg) -> Sub msg
port try_open_websocket : () -> Cmd msg
port recieve_websocket_data : (String -> msg) -> Sub msg
port send_metrics_request : () -> Cmd msg

type alias Flags = ()

type alias Metrics = { time: String, path: String }

type Model = WaitingForSocket
    | WaitingForInfo
    | Error String
    | Info Metrics

type Message = WebSocketChange String
    | WebSocketRecieve String
    | MainButtonClick

init : Flags -> (Model, Cmd Message)
init _ = (WaitingForSocket, try_open_websocket ())

view_title : String
view_title = "Loadstone Metrics"

view_loading : String -> List (Html Message)
view_loading message =
    [
        div [ id "container" ] [
            img [ src "/loading.gif" ] [],
            h2 [] [ text message ]
        ]
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

view_info : Metrics -> List (Html Message)
view_info metrics =
    [
        view_info_pane "Timing" "The time taken from power-on until the demo-app is booted into." metrics.time,
        view_info_pane "Boot path" "The path taken by loadstone when deciding how to boot." metrics.path
    ]

get_model_id : Model -> String
get_model_id model =
    case model of
        WaitingForSocket -> "loading"
        WaitingForInfo -> "loading"
        Info _ -> "info"
        Error _ -> "error"

get_model_main : Model -> List (Html Message)
get_model_main model =
    case model of
        WaitingForSocket -> view_loading "Waiting for response from remote server..."
        WaitingForInfo -> view_loading "Waiting for metrics from remote server..."
        Info metrics -> view_info metrics
        Error message -> view_error message

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

view : Model -> Document Message
view model =
    {
        title = view_title,
        body = view_body model
    }

failed_to_open_message : String
failed_to_open_message = "Failed to open a WebSocket connection to the " ++
    "server. This is most likely because the server failed to communicate" ++
    " with the board."

update_waiting_for_socket : Message -> (Model, Cmd Message)
update_waiting_for_socket message =
    case message of
        WebSocketChange "open" ->
            (WaitingForInfo, send_metrics_request ())
        WebSocketChange "closed" ->
            (Error failed_to_open_message, Cmd.none)
        _ ->
            (Error websocket_change_message, Cmd.none)

parse_time_metric : String -> Maybe String
parse_time_metric string =
    let
        prefix = "* Boot process took "
        suffix = " milliseconds."
        is_valid = String.startsWith prefix string
            && String.endsWith suffix string
        slice_start = String.length prefix
        slice_end = (String.length string) - (String.length suffix)
    in
    if is_valid then
        Just (String.slice slice_start slice_end string)
    else
        Nothing

parse_path_metric : String -> Maybe String
parse_path_metric string =
    let
        prefix = "* "
        slice_start = String.length prefix
        slice_end = String.length string
    in
    Just (String.slice slice_start slice_end string)

parse_metric_lines : String -> String -> Maybe Metrics
parse_metric_lines path_string time_string =
    case (parse_time_metric time_string, parse_path_metric path_string) of
        (Just time, Just path) -> Just { time = time ++ "ms", path = path }
        _ -> Nothing

parse_metrics : String -> Maybe Metrics
parse_metrics input =
    case String.lines input of
       _ :: second :: third :: _ -> parse_metric_lines second third
       _ -> Nothing

websocket_change_message : String
websocket_change_message = "The socket changed state unexpectedly. This " ++
    "most likely means the server crashed or failed to respond to a request" ++
    ", or you have lost internet connection."

bad_metrics_message : String
bad_metrics_message = "The remote server failed to provide meaningful " ++
    "metrics. Either the server temporarily returned incorrect data or the " ++
    "board is malfunctioning."

update_waiting_for_info : Message -> (Model, Cmd Message)
update_waiting_for_info message =
    case message of
        WebSocketChange _ ->
            (Error websocket_change_message, Cmd.none)
        WebSocketRecieve metrics ->
            case (parse_metrics metrics) of
            Just m -> (Info m, Cmd.none)
            Nothing -> (Error bad_metrics_message, Cmd.none)
        _ ->
            (WaitingForInfo, Cmd.none)

update_error : String -> Message -> (Model, Cmd Message)
update_error m message =
    case message of
       MainButtonClick -> (Error m, Browser.Navigation.reload)
       _ -> (Error m, Cmd.none)

update : Message -> Model -> (Model, Cmd Message)
update message model =
    case model of
        WaitingForSocket -> update_waiting_for_socket message
        WaitingForInfo -> update_waiting_for_info message
        Info _ -> (model, Cmd.none)
        Error m -> update_error m message

subscriptions : Model -> Sub Message
subscriptions _ =
    Sub.batch [
        report_websocket_state WebSocketChange,
        recieve_websocket_data WebSocketRecieve
    ]

main : Program Flags Model Message
main =
    document {
        init = init,
        view = view,
        update = update,
        subscriptions = subscriptions
    }
