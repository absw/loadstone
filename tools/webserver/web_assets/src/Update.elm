module Update exposing (update, make_metrics_request)

import Model exposing (..)
import Upload

import Http exposing (expectJson)
import Json.Decode as Decode
import File exposing (File)
import File.Select exposing (file)
import Bytes exposing (Bytes)
import File exposing (toBytes)
import Task

make_metrics_request : Cmd Message
make_metrics_request = Http.get
    {
        url = "/api/metrics",
        expect = expectJson MetricsRecieved decode_json_metrics
    }

decode_json_metrics : Decode.Decoder MetricsInfo
decode_json_metrics =
    Decode.map3 MetricsInfo
        (Decode.field "error" Decode.string)
        (Decode.field "time" Decode.string)
        (Decode.field "path" Decode.string)

update : Message -> Model -> (Model, Cmd Message)
update message model =
    case message of
        SwitchTab tab -> update_switch_tab tab model
        RetryMetricsDownload -> update_retry_metrics_download model
        MetricsRecieved result -> update_metrics_recieved result model
        SelectUploadFile file -> update_select_upload_file file model
        ConfirmUploadFile file -> update_confirm_upload_file file model
        FileConvertedToBytes bytes -> update_file_converted_to_bytes bytes model
        OpenFileSelectDialogue -> update_open_file_select_dialogue model
        UploadNotify notification -> update_upload_notification notification model

update_switch_tab : Tab -> Model -> (Model, Cmd Message)
update_switch_tab tab model =
    (
        { model | tab = tab },
        Cmd.none
    )

update_retry_metrics_download : Model -> (Model, Cmd Message)
update_retry_metrics_download model =
    (
        { model | info = InfoWaiting },
        make_metrics_request
    )

update_metrics_recieved : Result Http.Error MetricsInfo -> Model -> (Model, Cmd Message)
update_metrics_recieved result model =
    (
        { model | info = handle_recieved_info result },
        Cmd.none
    )

update_select_upload_file : File -> Model -> (Model, Cmd Message)
update_select_upload_file file model =
    (
        { model | upload = UploadFileSelected file },
        Cmd.none
    )

update_confirm_upload_file : File -> Model -> (Model, Cmd Message)
update_confirm_upload_file file model =
    (
        { model | upload = UploadInProgress file UploadWaitingOnBytes },
        Task.perform FileConvertedToBytes (toBytes file)
    )

update_file_converted_to_bytes : Bytes -> Model -> (Model, Cmd Message)
update_file_converted_to_bytes bytes model =
    case model.upload of
        UploadInProgress file UploadWaitingOnBytes ->
            (
                { model | upload = UploadInProgress file (UploadStarting bytes) },
                Upload.start bytes
            )
        _ -> (model, Cmd.none)

update_open_file_select_dialogue : Model -> (Model, Cmd Message)
update_open_file_select_dialogue model =
    (model, File.Select.file [] SelectUploadFile)

update_upload_notification : Upload.Notification -> Model -> (Model, Cmd Message)
update_upload_notification notification model =
    case model.upload of
        UploadInProgress file progress ->
            (
                { model | upload = update_upload_progress file progress notification },
                Cmd.none
            )
        _ -> (model, Cmd.none)

update_upload_progress : File -> UploadProgress -> Upload.Notification -> Upload
update_upload_progress file _ notification =
    case notification of
        Upload.UploadNotificationActive progress -> UploadInProgress file (Uploading progress)
        Upload.UploadNotificationFailed reason -> UploadInProgress file (UploadFailure reason)
        Upload.UploadNotificationDone -> UploadInProgress file UploadSuccess

handle_recieved_info : Result Http.Error MetricsInfo -> Info
handle_recieved_info result =
    case result of
        Ok info ->
            if info.error == "none" then
                InfoDisplay { time = info.time, path = info.path }
            else
                InfoError (get_info_error_message info.error)
        Err error ->
                InfoError (get_http_error_message error)

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
