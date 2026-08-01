using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.UnitTests.ValueObjects;

/// <summary>
/// Unit tests for Rating value object.
/// </summary>
public class RatingTests
{
    [Fact]
    public void From_ValidValue_ShouldCreateRating()
    {
        // Arrange & Act
        var rating = Rating.From(5);

        // Assert
        rating.Value.Should().Be(5);
    }

    [Fact]
    public void From_ZeroValue_ShouldCreateDefaultRating()
    {
        // Arrange & Act
        var rating = Rating.From(0);

        // Assert
        rating.Should().Be(Rating.Default);
        rating.IsRated.Should().BeFalse();
    }

    [Fact]
    public void From_ValueAboveMax_ShouldThrowException()
    {
        // Arrange & Act
        Action act = () => Rating.From(11);

        // Assert
        act.Should().Throw<ArgumentOutOfRangeException>();
    }

    [Fact]
    public void From_ValueBelowMin_ShouldThrowException()
    {
        // Arrange & Act
        Action act = () => Rating.From(-1);

        // Assert
        act.Should().Throw<ArgumentOutOfRangeException>();
    }

    [Fact]
    public void IsRated_ZeroRating_ShouldReturnFalse()
    {
        // Arrange
        var rating = Rating.From(0);

        // Assert
        rating.IsRated.Should().BeFalse();
    }

    [Fact]
    public void IsRated_NonZeroRating_ShouldReturnTrue()
    {
        // Arrange
        var rating = Rating.From(5);

        // Assert
        rating.IsRated.Should().BeTrue();
    }

    [Fact]
    public void ToString_Unrated_ShouldReturnUnrated()
    {
        // Arrange
        var rating = Rating.From(0);

        // Act
        var str = rating.ToString();

        // Assert
        str.Should().Be("Unrated");
    }

    [Fact]
    public void ToString_Rated_ShouldReturnFormattedString()
    {
        // Arrange
        var rating = Rating.From(8);

        // Act
        var str = rating.ToString();

        // Assert
        str.Should().Be("8/10");
    }

    [Fact]
    public void Equality_SameValue_ShouldBeEqual()
    {
        // Arrange
        var rating1 = Rating.From(5);
        var rating2 = Rating.From(5);

        // Assert
        rating1.Should().Be(rating2);
    }

    [Fact]
    public void ComparisonOperators_ShouldWorkCorrectly()
    {
        // Arrange
        var rating1 = Rating.From(5);
        var rating2 = Rating.From(8);

        // Assert
        (rating1 < rating2).Should().BeTrue();
        (rating1 > rating2).Should().BeFalse();
        (rating1 <= rating2).Should().BeTrue();
        (rating1 >= rating2).Should().BeFalse();
    }
}