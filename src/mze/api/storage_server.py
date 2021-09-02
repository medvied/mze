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

'''
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

'''

class StorageServer:
    pass


class StorageClient:
    pass
