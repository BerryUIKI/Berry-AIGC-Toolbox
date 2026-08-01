using BerryAIGen.Domain.Common;
using BerryAIGen.Domain.Events;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Domain.Entities;

/// <summary>
/// Represents a folder in the file system being watched for images.
/// </summary>
public sealed class Folder : AggregateRoot<FolderId>
{
    /// <summary>
    /// Gets the folder path.
    /// </summary>
    public FilePath Path { get; private set; }

    /// <summary>
    /// Gets the parent folder ID (null for root folders).
    /// </summary>
    public FolderId? ParentFolderId { get; private set; }

    /// <summary>
    /// Gets whether this folder is being watched for new images.
    /// </summary>
    public bool IsWatched { get; private set; }

    /// <summary>
    /// Gets whether to scan subfolders recursively.
    /// </summary>
    public bool Recursive { get; private set; }

    /// <summary>
    /// Gets the timestamp when this folder was last scanned.
    /// </summary>
    public DateTime? LastScannedAt { get; private set; }

    /// <summary>
    /// Gets the count of images in this folder.
    /// </summary>
    public int ImageCount { get; private set; }

    /// <summary>
    /// Gets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the last modification timestamp.
    /// </summary>
    public DateTime ModifiedAt { get; private set; }

    /// <summary>
    /// Private constructor for EF Core.
    /// </summary>
    private Folder() { }

    /// <summary>
    /// Creates a new Folder.
    /// </summary>
    /// <param name="path">The folder path.</param>
    /// <param name="recursive">Whether to scan recursively.</param>
    /// <param name="parentFolderId">Optional parent folder ID.</param>
    /// <returns>A new Folder instance.</returns>
    public static Folder Create(
        FilePath path,
        bool recursive = true,
        FolderId? parentFolderId = null)
    {
        if (path is null)
            throw new ArgumentNullException(nameof(path));

        var folder = new Folder
        {
            Id = FolderId.New(),
            Path = path,
            Recursive = recursive,
            ParentFolderId = parentFolderId,
            IsWatched = true,
            ImageCount = 0,
            CreatedAt = DateTime.UtcNow,
            ModifiedAt = DateTime.UtcNow
        };

        folder.RaiseDomainEvent(new FolderCreatedEvent(folder.Id, path));
        return folder;
    }

    /// <summary>
    /// Marks this folder as being watched for changes.
    /// </summary>
    public void StartWatching()
    {
        if (!IsWatched)
        {
            IsWatched = true;
            ModifiedAt = DateTime.UtcNow;
        }
    }

    /// <summary>
    /// Stops watching this folder for changes.
    /// </summary>
    public void StopWatching()
    {
        if (IsWatched)
        {
            IsWatched = false;
            ModifiedAt = DateTime.UtcNow;
        }
    }

    /// <summary>
    /// Updates the recursive scanning setting.
    /// </summary>
    /// <param name="recursive">Whether to scan recursively.</param>
    public void SetRecursive(bool recursive)
    {
        Recursive = recursive;
        ModifiedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Records that a scan was completed.
    /// </summary>
    /// <param name="imageCount">The count of images found.</param>
    public void RecordScan(int imageCount)
    {
        if (imageCount < 0)
            throw new ArgumentException("Image count cannot be negative.", nameof(imageCount));

        LastScannedAt = DateTime.UtcNow;
        ImageCount = imageCount;
        ModifiedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Increments the image count.
    /// </summary>
    public void IncrementImageCount()
    {
        ImageCount++;
        ModifiedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Decrements the image count.
    /// </summary>
    public void DecrementImageCount()
    {
        if (ImageCount > 0)
        {
            ImageCount--;
            ModifiedAt = DateTime.UtcNow;
        }
    }
}