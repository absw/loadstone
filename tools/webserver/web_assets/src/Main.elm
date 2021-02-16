port module Main exposing (main)

import Browser exposing (Document, document)
import Html exposing (Html, input, text, button, div, span, main_, h1, h2, pre, dl, dt, dd)
import Html.Attributes exposing (class, id, value)
import Html.Events exposing (onClick, onInput)

port report_websocket_state : (String -> msg) -> Sub msg
port try_open_websocket : () -> Cmd msg
port send_websocket_data : String -> Cmd msg
port recieve_websocket_data : (String -> msg) -> Sub msg
port send_metrics_request : () -> Cmd msg

type alias Flags = ()

type alias Terminal = { input: String, responses: String }

type alias Metrics = { time: String, path: String }

type Model = WaitingForSocket
    | BadSocket String
    | WaitingForInfo
    | Info Metrics
    | Running Terminal

type Message = InputSubmit
    | InputChange String
    | WebSocketChange String
    | WebSocketRecieve String

init : Flags -> (Model, Cmd Message)
init _ = (WaitingForSocket, try_open_websocket ())

view_title : String
view_title = "Loadstone Image Loader"

view_running : Terminal -> List (Html Message)
view_running terminal =
    [
        main_ [ id "terminal" ] [
            pre [ id "response-container" ] [ text terminal.responses ],
            div [ id "input-container"] [
                input [ onInput InputChange, value terminal.input ] [],
                span [ class "spacer" ] [],
                button [ onClick InputSubmit ] [ text "\u{23CE}" ]
            ]
        ]
    ]

view_info : Metrics -> List (Html Message)
view_info metrics =
    [
        h1 [] [ text "Loadstone metrics" ],
        main_ [ id "metrics" ] [
            dl [] [
                dt [] [ text "Timing"],
                dd [] [ text (metrics.time ++ "ms") ],
                dt [] [ text "Boot path" ],
                dd [] [ text metrics.path ]
            ]
        ]
    ]

view_message : String -> List (Html Message)
view_message message =
    [
        main_ [ id "solo-message" ] [
            h1 [] [ text message ]
        ]
    ]

view_body : Model -> List (Html Message)
view_body model =
    case model of
        WaitingForSocket -> view_message "Waiting for response from remote server..."
        WaitingForInfo -> view_message "Waiting for metrics from remote server..."
        Info metrics -> view_info metrics
        Running terminal -> view_running terminal
        BadSocket reason -> view_message reason

view : Model -> Document Message
view model =
    {
        title = view_title,
        body = view_body model
    }

update_terminal_on_recieve : Terminal -> String -> Terminal
update_terminal_on_recieve terminal line =
    { terminal | responses = terminal.responses ++ line }

update_running : Message -> Terminal -> (Model, Cmd Message)
update_running message terminal =
    case message of
        InputChange new_value ->
            (Running { terminal | input = new_value }, Cmd.none)
        InputSubmit ->
            (Running { terminal | input = "" }, send_websocket_data terminal.input )
        WebSocketChange _ ->
            (BadSocket "The socket changed state unexpectedly.", Cmd.none)
        WebSocketRecieve line ->
            (Running (update_terminal_on_recieve terminal line), Cmd.none)

update_waiting_for_socket : Message -> (Model, Cmd Message)
update_waiting_for_socket message =
    case message of
        WebSocketChange "open" ->
            (WaitingForInfo, send_metrics_request ())
--          (Running { input = "", responses = "" }, Cmd.none)
        WebSocketChange "closed" ->
            (BadSocket "The remote server failed to open a web socket.", Cmd.none)
        _ ->
            (WaitingForSocket, Cmd.none)

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
        (Just time, Just path) -> Just { time = time, path = path }
        _ -> Nothing

parse_metrics : String -> Maybe Metrics
parse_metrics input =
    case String.lines input of
       _ :: second :: third :: _ -> parse_metric_lines second third
       _ -> Nothing

update_waiting_for_info : Message -> (Model, Cmd Message)
update_waiting_for_info message =
    case message of
        WebSocketChange _ ->
            (BadSocket "The socket changed state unexpectedly.", Cmd.none)
        WebSocketRecieve metrics ->
            case (parse_metrics metrics) of
            Just m -> (Info m, Cmd.none)
            Nothing -> (BadSocket "The remote server failed to provide meaningful metrics.", Cmd.none)
        _ ->
            (WaitingForInfo, Cmd.none)

update : Message -> Model -> (Model, Cmd Message)
update message model =
    case model of
        WaitingForSocket -> update_waiting_for_socket message
        WaitingForInfo -> update_waiting_for_info message
        Info metrics -> (model, Cmd.none)
        Running terminal -> update_running message terminal
        BadSocket _ -> (model, Cmd.none)

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
