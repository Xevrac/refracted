using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlayerExperienceChange
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlayerExperienceChange); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlayerExperienceChange)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize Rank
            s.Write(value.Rank);
            //  Serialize RankProgress
            s.Write(value.RankProgress);
            //  Serialize RankGoal
            s.Write(value.RankGoal);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlayerExperienceChange)) as Rts.CnC.Messages.Client.PlayerExperienceChange;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize Rank
            s.Read(out value.Rank);
            //  Deserialize RankProgress
            s.Read(out value.RankProgress);
            //  Deserialize RankGoal
            s.Read(out value.RankGoal);

            return value;
        }
        
    }
}
