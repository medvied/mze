#!/usr/bin/env python3
#
# Copyright 2021 Maksym Medvied
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""
StorageServer API is based on AWS S3 API.

https://docs.aws.amazon.com/AmazonS3/latest/API/API_Operations_Amazon_Simple_Storage_Service.html

Here is the full list of S3 API functions, their classification and which part
is implemented or extended.

===========================================  ==================================
Function                                     Comment
===========================================  ==================================
AbortMultipartUpload                         multipart
CompleteMultipartUpload                      multipart
CopyObject                                   object, copy
CreateBucket                                 bucket
CreateMultipartUpload                        multipart
DeleteBucket                                 bucket
DeleteBucketAnalyticsConfiguration           -
DeleteBucketCors                             -
DeleteBucketEncryption                       -
DeleteBucketIntelligentTieringConfiguration  -
DeleteBucketInventoryConfiguration           -
DeleteBucketLifecycle                        -
DeleteBucketMetricsConfiguration             -
DeleteBucketOwnershipControls                -
DeleteBucketPolicy                           -
DeleteBucketReplication                      -
DeleteBucketTagging                          -
DeleteBucketWebsite                          -
DeleteObject                                 object
DeleteObjects                                object
DeleteObjectTagging                          -
DeletePublicAccessBlock                      -
GetBucketAccelerateConfiguration             -
GetBucketAcl                                 -
GetBucketAnalyticsConfiguration              -
GetBucketCors                                -
GetBucketEncryption                          -
GetBucketIntelligentTieringConfiguration     -
GetBucketInventoryConfiguration              -
GetBucketLifecycle                           -
GetBucketLifecycleConfiguration              -
GetBucketLocation                            -
GetBucketLogging                             -
GetBucketMetricsConfiguration                -
GetBucketNotification                        -
GetBucketNotificationConfiguration           -
GetBucketOwnershipControls                   -
GetBucketPolicy                              -
GetBucketPolicyStatus                        -
GetBucketReplication                         -
GetBucketRequestPayment                      -
GetBucketTagging                             -
GetBucketVersioning                          -
GetBucketWebsite                             -
GetObject                                    object
GetObjectAcl                                 -
GetObjectLegalHold                           -
GetObjectLockConfiguration                   -
GetObjectRetention                           -
GetObjectTagging                             -
GetObjectTorrent                             -
GetPublicAccessBlock                         -
HeadBucket                                   bucket
HeadObject                                   object
ListBucketAnalyticsConfigurations            -
ListBucketIntelligentTieringConfigurations   -
ListBucketInventoryConfigurations            -
ListBucketMetricsConfigurations              -
ListBuckets                                  bucket
ListMultipartUploads                         multipart
ListObjects                                  bucket
ListObjectsV2                                bucket
ListObjectVersions                           object, versioning
ListParts                                    multipart
PutBucketAccelerateConfiguration             -
PutBucketAcl                                 -
PutBucketAnalyticsConfiguration              -
PutBucketCors                                -
PutBucketEncryption                          -
PutBucketIntelligentTieringConfiguration     -
PutBucketInventoryConfiguration              -
PutBucketLifecycle                           -
PutBucketLifecycleConfiguration              -
PutBucketLogging                             -
PutBucketMetricsConfiguration                -
PutBucketNotification                        -
PutBucketNotificationConfiguration           -
PutBucketOwnershipControls                   -
PutBucketPolicy                              -
PutBucketReplication                         -
PutBucketRequestPayment                      -
PutBucketTagging                             -
PutBucketVersioning                          -
PutBucketWebsite                             -
PutObject                                    object
PutObjectAcl                                 -
PutObjectLegalHold                           -
PutObjectLockConfiguration                   -
PutObjectRetention                           -
PutObjectTagging                             -
PutPublicAccessBlock                         -
RestoreObject                                -
SelectObjectContent                          -
UploadPart                                   multipart
UploadPartCopy                               multipart, copy
WriteGetObjectResponse                       -
===========================================  ==================================

Categories

Bucket

- CreateBucket
- DeleteBucket
- HeadBucket
- ListBuckets
- ListObjects
- ListObjectsV2

Object

- DeleteObject
- DeleteObjects
- GetObject
- HeadObject
- ListObjectVersions
- PutObject

Multipart

- AbortMultipartUpload
- CompleteMultipartUpload
- CreateMultipartUpload
- ListMultipartUploads
- ListParts
- UploadPart

Copy

- CopyObject
- UploadPartCopy

TODO

- async/await

"""

import uuid
import pathlib

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional, Any


@dataclass
class BlobId:
    bid: uuid.UUID


@dataclass
class BlobInfo:
    size: int
    info: dict[str, str]


@dataclass
class BlobData:
    """
    At least one of the fields should be something other than None.
    """
    data: Optional[bytes]
    path: Optional[pathlib.Path]


class Storage(ABC):
    """
    Subclasses of this class implement the abstract methods.
    In combination with StorageServer and StorageClient this allows to separate
    network transport from Storage implementation.

    Example:
    #. Make StorageDir(Storage) and implement abstract methods that are using
      files and directories to store the data.
    #. Make StorageServerHTTP(StorageServer) which receives requests over HTTP
       and calls the abstract methods to execute them.
    #. Make StorageDirHTTP(StorageServerHTTP, StorageDir). This would join both
       transport and implementation.
    #. Make StorageClientHTTP(StorageClient) which implements abstract methods
       of Storage by making requests to StorageServerHTTP over HTTP. This would
       allow to use StorageServerHTTP over the network.
    #. Make StorageClientDir(StorageClient, StorageDir). It would be a Storage
       Client which uses local directories and files to store the data.
    """
    @abstractmethod
    def init(self, cfg: dict[str, Any]) -> None:
        pass

    @abstractmethod
    def fini(self) -> None:
        pass

    @abstractmethod
    def create(self, cfg: dict[str, Any]) -> None:
        """
        Implies init().
        """
        pass

    @abstractmethod
    def destroy(self) -> None:
        """
        Implies fini().
        Assumption: there are no blobs left.
        """
        pass

    @abstractmethod
    def fsck(self) -> None:
        pass

    @abstractmethod
    def get(self, bids: list[BlobId]) -> \
            list[Optional[tuple[BlobInfo, BlobData]]]:
        """
        Returns None if there is no such blob.
        Returns filename or bytes in case if there is such blob.
        """
        pass

    @abstractmethod
    def put(self, blobs: list[tuple[BlobId, BlobInfo, BlobData]]) -> \
            list[BlobInfo]:
        pass

    @abstractmethod
    def head(self, bids: list[BlobId]) -> list[Optional[BlobInfo]]:
        pass

    @abstractmethod
    def catalog(self) -> list[tuple[BlobId, BlobInfo]]:
        pass

    @abstractmethod
    def delete(self, bids: list[BlobId]) -> list[Optional[BlobInfo]]:
        pass


class StorageServer(Storage):
    '''
    Subclasses of this class call abstract methods of Storage.
    '''
    pass


class StorageClient(Storage):
    '''
    Subclasses of this class implement abstract methods of Storage.
    '''
    pass
