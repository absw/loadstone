port module Upload exposing (start, notify, switch_notification, Notification(..))

import Json.Decode as Decode
import Bytes exposing (Bytes)
import Base64

switch_notification : (Float -> a) -> (String -> a) -> a -> Notification -> a
switch_notification active failed done notification =
    case notification of
        UploadNotificationActive f -> active f
        UploadNotificationFailed s -> failed s
        UploadNotificationDone -> done

start : Bytes -> Cmd message
start data = data
    |> Base64.fromBytes
    |> Maybe.withDefault ""
    |> upload_start

notify : (Notification -> message) -> Sub message
notify user_message = Sub.map user_message (upload_notify decode_notification)

type alias RawNotification = { done: Bool, progress: Float, error: String }
type Notification = UploadNotificationActive Float
    | UploadNotificationFailed String
    | UploadNotificationDone

raw_notification_decoder : Decode.Decoder RawNotification
raw_notification_decoder = Decode.map3 RawNotification
    (Decode.field "done" Decode.bool)
    (Decode.field "progress" Decode.float)
    (Decode.field "error" Decode.string)

decode_raw_notification : Decode.Value -> Result Decode.Error RawNotification
decode_raw_notification value = Decode.decodeValue raw_notification_decoder value

decode_notification : Decode.Value -> Notification
decode_notification value =
    case (decode_raw_notification value) of
        Err _ -> UploadNotificationFailed "Failed to parse internal upload notification."
        Ok raw ->
            case (raw.done, raw.error) of
                (False, _) -> UploadNotificationActive raw.progress
                (True, "") -> UploadNotificationDone
                (True, e) -> UploadNotificationFailed e

port upload_start : String -> Cmd message
port upload_notify : (Decode.Value -> message) -> Sub message
