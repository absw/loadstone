module View exposing (view)

import Model exposing (..)
import Pane exposing (..)

import Browser exposing (Document)
import Html exposing (..)
import Html.Attributes exposing (class, id, src, href)
import Html.Events exposing (onClick)
import File exposing (File)
import File.Select exposing (file)
import Bytes exposing (Bytes)

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
        header [] (view_header),
        main_ [ id (get_model_id model) ] (
            view_main model
        )
    ]

view_header : List (Html Message)
view_header =
    [
        h1 [] [ text view_title ],
        div [ id "header-spacer" ] [],
        view_nav_link InfoTab "Metrics",
        view_nav_link UploadTab "Upload"
    ]

view_nav_link : Tab -> String -> Html Message
view_nav_link tab tab_name =
    a [ class "nav-link", href "#", onClick (SwitchTab tab) ] [ text tab_name ]

get_model_id : Model -> String
get_model_id model =
    case model.tab of
        InfoTab -> get_info_model_id model.info
        UploadTab -> get_upload_model_id model.upload

get_info_model_id : Info -> String
get_info_model_id info =
    case info of
        InfoWaiting -> "loading"
        InfoError _ -> "error"
        InfoDisplay _ -> "info"

get_upload_model_id : Upload -> String
get_upload_model_id _ = "info"

view_main : Model -> List (Html Message)
view_main model =
    case model.tab of
        InfoTab -> view_info_main model.info
        UploadTab -> view_upload_main model.upload

view_info_main : Info -> List (Html Message)
view_info_main info =
    case info of
        InfoWaiting -> view_info_waiting "Waiting for metrics from remote server..."
        InfoError error -> view_info_error error
        InfoDisplay metrics -> view_info_display metrics

view_info_waiting : String -> List (Html Message)
view_info_waiting message =
    [
        div [ id "container" ] [
            img [ src "/loading.gif" ] [],
            h2 [] [ text message ]
        ]
    ]

view_info_display : Metrics -> List (Html Message)
view_info_display metrics =
    [
        info_pane "Timing"
            "The time taken from power-on until the demo-app is booted into." metrics.time,
        info_pane "Boot path"
            "The path taken by loadstone when deciding how to boot." metrics.path
    ]

view_info_error : String -> List (Html Message)
view_info_error message =
    [
        div [ id "container" ] [
            h2 [] [ text "Failed to retrieve metrics" ],
            p [] [ text message ],
            button [ onClick RetryMetricsDownload ] [ text "Reload" ]
        ]
    ]

view_upload_main : Upload -> List (Html Message)
view_upload_main upload =
    case upload of
        UploadInitial -> view_upload_initial
        UploadFileSelected file -> view_upload_file_selected file
        UploadInProgress file progress -> view_upload_in_progress file progress

view_upload_initial : List (Html Message)
view_upload_initial =
    [
        file_pane "Select file" "Select a firmware image to upload to the board."
            Nothing OpenFileSelectDialogue
    ]

view_upload_file_selected : File -> List (Html Message)
view_upload_file_selected file = view_upload_file_selected_x file False

view_upload_file_selected_x : File -> Bool -> List (Html Message)
view_upload_file_selected_x file b =
    [
        file_pane "Select file" "Select a firmware image to upload to the board."
            (Just file) OpenFileSelectDialogue,
        button_pane "Confirm file" "Click below to confirm this image and begin the upload."
            "Upload" b (ConfirmUploadFile file)
    ]

view_upload_in_progress : File -> UploadProgress -> List (Html Message)
view_upload_in_progress file progress =
    case progress of
        UploadWaitingOnBytes -> view_upload_starting file
        UploadStarting _ -> view_upload_starting file
        Uploading _ _ -> []
        UploadFailure _ -> []
        UploadSuccess -> []

view_upload_starting : File -> List (Html Message)
view_upload_starting file =
    [
        notice_pane "Starting upload" "Waiting for a connection to the server..."
    ]
    |> List.append (view_upload_file_selected_x file True)
