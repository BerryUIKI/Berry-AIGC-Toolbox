using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.Events;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Domain.Tests.Entities;

/// <summary>
/// Unit tests for Image aggregate root.
/// </summary>
public class ImageTests
{
    [Fact]
    public void Create_ValidParameters_ShouldCreateImage()
    {
        // Arrange
        var path = FilePath.From(@"C:\Images\test.png");
        var hash = Hash.From("ABC123DEF456", "SHA256");
        var dimensions = ImageDimensions.From(1920, 1080);
        var folderId = FolderId.New();

        // Act
        var image = Image.Create(path, hash, dimensions, 1024000, folderId, "PNG");

        // Assert
        image.Should().NotBeNull();
        image.Path.Should().Be(path);
        image.Hash.Should().Be(hash);
        image.Dimensions.Should().Be(dimensions);
        image.FileSize.Should().Be(1024000);
        image.FolderId.Should().Be(folderId);
        image.ImageType.Should().Be("PNG");
        image.Rating.Should().Be(Rating.Default);
        image.IsFavorite.Should().BeFalse();
        image.IsNSFW.Should().BeFalse();
        image.IsAvailable.Should().BeTrue();
        image.TagIds.Should().BeEmpty();
        image.AlbumIds.Should().BeEmpty();
    }

    [Fact]
    public void Create_ShouldRaiseImageCreatedEvent()
    {
        // Arrange
        var path = FilePath.From(@"C:\Images\test.png");
        var hash = Hash.From("ABC123DEF456", "SHA256");
        var dimensions = ImageDimensions.From(1920, 1080);
        var folderId = FolderId.New();

        // Act
        var image = Image.Create(path, hash, dimensions, 1024000, folderId, "PNG");

        // Assert
        image.DomainEvents.Should().ContainSingle();
        image.DomainEvents.First().Should().BeOfType<ImageCreatedEvent>();
    }

    [Fact]
    public void SetRating_ValidRating_ShouldUpdateRating()
    {
        // Arrange
        var image = CreateTestImage();
        var rating = Rating.From(8);

        // Act
        image.SetRating(rating);

        // Assert
        image.Rating.Should().Be(rating);
    }

    [Fact]
    public void SetRating_ShouldRaiseImageRatedEvent()
    {
        // Arrange
        var image = CreateTestImage();
        var rating = Rating.From(8);

        // Act
        image.SetRating(rating);

        // Assert
        image.DomainEvents.Should().Contain(e => e is ImageRatedEvent);
    }

    [Fact]
    public void MarkAsFavorite_ShouldSetIsFavoriteToTrue()
    {
        // Arrange
        var image = CreateTestImage();

        // Act
        image.MarkAsFavorite();

        // Assert
        image.IsFavorite.Should().BeTrue();
    }

    [Fact]
    public void MarkAsFavorite_ShouldRaiseImageMarkedFavoriteEvent()
    {
        // Arrange
        var image = CreateTestImage();

        // Act
        image.MarkAsFavorite();

        // Assert
        image.DomainEvents.Should().Contain(e => e is ImageMarkedFavoriteEvent);
    }

    [Fact]
    public void RemoveFromFavorites_ShouldSetIsFavoriteToFalse()
    {
        // Arrange
        var image = CreateTestImage();
        image.MarkAsFavorite();

        // Act
        image.RemoveFromFavorites();

        // Assert
        image.IsFavorite.Should().BeFalse();
    }

    [Fact]
    public void AddTag_ShouldAddTagToCollection()
    {
        // Arrange
        var image = CreateTestImage();
        var tagId = TagId.New();

        // Act
        image.AddTag(tagId);

        // Assert
        image.TagIds.Should().Contain(tagId);
    }

    [Fact]
    public void AddTag_DuplicateTag_ShouldNotAddAgain()
    {
        // Arrange
        var image = CreateTestImage();
        var tagId = TagId.New();
        image.AddTag(tagId);

        // Act
        image.AddTag(tagId);

        // Assert
        image.TagIds.Should().HaveCount(1);
    }

    [Fact]
    public void RemoveTag_ExistingTag_ShouldRemoveFromCollection()
    {
        // Arrange
        var image = CreateTestImage();
        var tagId = TagId.New();
        image.AddTag(tagId);

        // Act
        image.RemoveTag(tagId);

        // Assert
        image.TagIds.Should().NotContain(tagId);
    }

    [Fact]
    public void AddToAlbum_ShouldAddAlbumToCollection()
    {
        // Arrange
        var image = CreateTestImage();
        var albumId = AlbumId.New();

        // Act
        image.AddToAlbum(albumId);

        // Assert
        image.AlbumIds.Should().Contain(albumId);
    }

    private static Image CreateTestImage()
    {
        var path = FilePath.From(@"C:\Images\test.png");
        var hash = Hash.From("ABC123DEF456", "SHA256");
        var dimensions = ImageDimensions.From(1920, 1080);
        var folderId = FolderId.New();

        return Image.Create(path, hash, dimensions, 1024000, folderId, "PNG");
    }
}