module Pane exposing (PaneStatus(..), progress_pane, notice_pane, info_pane, file_pane, button_pane)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import File exposing (File)

type PaneStatus = PaneDefault
    | PaneSuccess
    | PaneFailure

pane_status_class : PaneStatus -> String
pane_status_class status =
    case status of
        PaneDefault -> "pane-default"
        PaneSuccess -> "pane-success"
        PaneFailure -> "pane-failure"

progress_pane : String -> String -> PaneStatus -> Float -> Html msg
progress_pane title description status progress =
    pane title description status [
        div [ class "progress-bar-container" ] [
            div [ class "progress-bar", style "width" (percentage_to_string progress) ] []
        ]
    ]

notice_pane : String -> String -> Html msg
notice_pane title description =
    pane title description PaneDefault []

info_pane : String -> String -> String -> Html msg
info_pane title description value =
    pane title description PaneDefault [ div [ class "pane-value" ] [ text value ] ]

file_pane : String -> String -> Maybe File -> msg -> Html msg
file_pane title description file message =
    let
        status = (if file == Nothing then PaneFailure else PaneSuccess)
        value_text = (Maybe.withDefault "No file selected" (Maybe.map get_file_summary file))
        make_full_pane = pane title description status
    in
    make_full_pane [
        div [ class "pane-controls" ] [
            span [ class "pane-value" ] [ text value_text ],
            a [ href "#", class "file-select-button", onClick message ] [ text "Select File" ]
        ]
    ]

button_pane : String -> String -> String -> Bool -> msg -> Html msg
button_pane title description content is_okay message =
    let
        status = (if is_okay then PaneSuccess else PaneFailure)
        make_full_pane = pane title description status
    in
    make_full_pane [
        a [ href "#", class "pane-button", onClick message ] [ text content ]
    ]

pane : String -> String -> PaneStatus -> List (Html msg) -> Html msg
pane title description status inner =
    div [ class "pane", class (pane_status_class status) ] (
        (div [ class "pane-top" ] [
            h2 [] [ text title ],
            span [] [ text description ]
        ]) :: inner
    )

get_file_summary : File -> String
get_file_summary file = (File.name file) ++ " (" ++ (bytes_to_string (File.size file) ++ ")")

bytes_to_string : Int -> String
bytes_to_string n =
    let
        bytes = n
        kilobytes = round (toFloat n / 1024.0)
        megabytes = round (toFloat n / 1024.0 / 1024.0)
    in
    if n < 1024 then
        String.fromInt bytes ++ "B"
    else if n < (1024 * 1024) then
        String.fromInt kilobytes ++ "KB"
    else
        String.fromInt megabytes ++ "MB"

percentage_to_string : Float -> String
percentage_to_string n = n
    |> (*) 100
    |> String.fromFloat
    |> \a -> String.append a "%"
