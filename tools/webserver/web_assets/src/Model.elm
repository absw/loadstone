module Model exposing (..)

import Upload

import Bytes exposing (Bytes)
import File exposing (File)
import Http

type alias Flags = ()

type alias Metrics = { time: String, path: String }

type alias MetricsInfo = { error: String, time: String, path: String }

type Tab = InfoTab
    | UploadTab

type Info = InfoWaiting
    | InfoError String
    | InfoDisplay Metrics

type UploadProgress = UploadWaitingOnBytes
    | UploadStarting Bytes
    | Uploading Float
    | UploadFinalising
    | UploadFailure String
    | UploadSuccess

type Upload = UploadInitial
    | UploadFileSelected File
    | UploadInProgress File UploadProgress

type alias Model = { tab: Tab, info: Info, upload: Upload }

type Message = SwitchTab Tab
    | RetryMetricsDownload
    | MetricsRecieved (Result Http.Error MetricsInfo)
    | SelectUploadFile File
    | ConfirmUploadFile File
    | FileConvertedToBytes Bytes
    | OpenFileSelectDialogue
    | UploadNotify Upload.Notification

default : Model
default =
    {
        tab = InfoTab,
        info = InfoWaiting,
        upload = UploadInitial
    }
