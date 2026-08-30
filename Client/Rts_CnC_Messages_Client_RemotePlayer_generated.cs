using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RemotePlayer
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RemotePlayer); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RemotePlayer)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize PlayerBlazeId
            s.Write(value.PlayerBlazeId);
            //  Serialize PlayerType
            s.Write(value.PlayerType);
            //  Serialize AllegianceLevel
            s.Write(value.AllegianceLevel);
            //  Serialize Faction
            s.Write(value.Faction);
            //  Serialize General
            s.Write(value.General);
            //  Serialize SkillTreeActive
            s.Write(value.SkillTreeActive);
            //  Serialize TeamId
            s.Write(value.TeamId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RemotePlayer)) as Rts.CnC.Messages.Client.RemotePlayer;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize PlayerBlazeId
            s.Read(out value.PlayerBlazeId);
            //  Deserialize PlayerType
            s.Read(out value.PlayerType);
            //  Deserialize AllegianceLevel
            s.Read(out value.AllegianceLevel);
            //  Deserialize Faction
            s.Read(out value.Faction);
            //  Deserialize General
            s.Read(out value.General);
            //  Deserialize SkillTreeActive
            s.Read(out value.SkillTreeActive);
            //  Deserialize TeamId
            s.Read(out value.TeamId);

            return value;
        }
        
    }
}
