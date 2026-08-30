using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_CreateGame
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.GameSetup.CreateGame); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.GameSetup.CreateGame)obj;
            //  Serialize RequestId
            s.Write(ref value.RequestId);
            //  Serialize MapUri
            s.Write(value.MapUri);
            //  Serialize GameId
            s.Write(ref value.GameId);
            //  Serialize array Players
            Rts.Serialization.Reference.Write(s, value.Players, () =>
            {
                s.WriteVarInt32(value.Players.Length);
                for(int i = 0 ; i < value.Players.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_PlayerInfo.Serializer.Serialize(s, value.Players[i]);
                }
            });
            //  Serialize Attributes
            s.Write(value.Attributes);
            //  Serialize Options
            s.WriteEnum(value.Options);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.GameSetup.CreateGame)) as Rts.CnC.Messages.GameSetup.CreateGame;
            //  Deserialize RequestId
            s.Read(out value.RequestId);
            //  Deserialize MapUri
            s.Read(out value.MapUri);
            //  Deserialize GameId
            s.Read(out value.GameId);
            //  Deserialize array Players
            Rts.Serialization.Reference.Read(s, out value.Players, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.GameSetup.PlayerInfo[] tmp = new Rts.CnC.Messages.GameSetup.PlayerInfo[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_PlayerInfo.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });
            //  Deserialize Attributes
            s.Read(out value.Attributes);
            //  Deserialize Options
            s.ReadEnum(out value.Options);

            return value;
        }
        
    }
}
