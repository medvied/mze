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

import json
import uuid
import pathlib

from collections.abc import Sequence
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional, Any

from .cli import CLI
from .service import ServiceServer, ServiceClient


@dataclass
class BlobId:
    bid: uuid.UUID


@dataclass
class BlobInfo:
    size: int


@dataclass
class BlobData:
    """
    At least one of the fields should be something other than None.
    TODO add invariant() to check that exactly one of files is set
    """
    data: Optional[bytes]
    path: Optional[pathlib.Path]


class Storage(ABC):
    """
    Subclasses of this class implement or use the abstract methods.
    Combination of StorageServer and StorageClient allows to separate
    network transport from the actual implementation.

    Example:
    #. Make StorageClientDir(StorageClient) and implement abstract methods that
       are using files and directories to store the data.
    #. Make StorageServerHTTP(StorageServer) which receives requests over HTTP
       and calls the abstract methods to execute them.
    #. Make StorageServerHTTPClientDir(StorageServerHTTP, StorageClientDir).
       This would join both transport and implementation.
    #. Make StorageClientHTTP(StorageClient) which implements abstract methods
       of Storage by making requests to StorageServerHTTP over HTTP. This would
       allow to use StorageServerHTTP over the network.
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
    def get(self, bids: Sequence[BlobId]) -> list[Optional[BlobData]]:
        """
        Returns None if there is no such blob.
        Returns filename or bytes in case if there is such blob.
        """
        pass

    @abstractmethod
    def put(self, blobs: Sequence[tuple[Optional[BlobId], BlobData]]) -> \
            list[tuple[BlobId, BlobInfo]]:
        pass

    @abstractmethod
    def head(self, bids: Sequence[BlobId]) -> list[Optional[BlobInfo]]:
        pass

    @abstractmethod
    def catalog(self) -> list[tuple[BlobId, BlobInfo]]:
        pass

    @abstractmethod
    def delete(self, bids: Sequence[BlobId]) -> list[Optional[BlobInfo]]:
        pass


class StorageServer(Storage, ServiceServer, CLI):
    """
    Subclasses of this class call abstract methods of Storage.
    """
    pass


class StorageClient(Storage, ServiceClient, CLI):
    cmds: str
    blob_id: Optional[uuid.UUID]
    filename_in: str
    filename_out: str
    init_cfg: dict[str, Any]
    create_cfg: dict[str, Any]
    """
    Subclasses of this class implement abstract methods of Storage.
    """
    def parse_cfg_argv_environ(self, cfg: dict[str, str], argv: list[str],
                               environ: dict[str, str]) -> None:
        self.cmds = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_CMD',
            argv_flags=['cmd'],
            argv_kwargs={'nargs': '*'})  # make it optional
        self.blob_id = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_BLOB_ID',
            argv_flags=['--blob-id'],
            argv_kwargs={'type': uuid.UUID, 'default': None})
        self.filename_in = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_FILENAME_IN',
            argv_flags=['--filename-in'],
            argv_kwargs={'type': str, 'default': '-'})
        self.filename_out = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_FILENAME_OUT',
            argv_flags=['--filename-out'],
            argv_kwargs={'type': str, 'default': '-'})
        self.init_cfg = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_INIT_CFG',
            argv_flags=['--init-cfg'],
            argv_kwargs={'type': json.loads, 'default': {}})
        self.create_cfg = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_CREATE_CFG',
            argv_flags=['--create-cfg'],
            argv_kwargs={'type': json.loads, 'default': {}})

    def run(self) -> None:
        # TODO better error message
        assert self.server_url is not None, self.cmds
        for cmd in self.cmds:
            # TODO better error message
            assert cmd in ['init', 'fini', 'create', 'destroy', 'fsck',
                           'get', 'put', 'head', 'catalog', 'delete'], \
                (cmd, self.cmds)
        # TODO use Structural Pattern Matching in Python 3.10
        for cmd in self.cmds:
            if cmd == 'init':
                self.init(self.init_cfg)
            elif cmd == 'fini':
                self.fini()
            elif cmd == 'create':
                self.create(self.create_cfg)
            elif cmd == 'destroy':
                self.destroy()
            elif cmd == 'fsck':
                self.fsck()
            else:
                print(f'{self.cmds=} {cmd=}')
                raise NotImplementedError
