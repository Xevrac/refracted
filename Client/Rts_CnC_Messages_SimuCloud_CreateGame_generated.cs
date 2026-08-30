using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_SimuCloud_CreateGame
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.SimuCloud.CreateGame); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.SimuCloud.CreateGame)obj;
            //  Serialize MapName
            s.Write(value.MapName);
            //  Serialize DirPath
            s.Write(value.DirPath);
            //  Serialize array Info
            Rts.Serialization.Reference.Write(s, value.Info, () =>
            {
                s.WriteVarInt32(value.Info.Length);
                for(int i = 0 ; i < value.Info.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_PlayerInfo.Serializer.Serialize(s, value.Info[i]);
                }
            });
            //  Serialize GameId
            s.Write(ref value.GameId);
            //  Serialize Options
            s.WriteEnum(value.Options);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.SimuCloud.CreateGame)) as Rts.CnC.Messages.SimuCloud.CreateGame;
            //  Deserialize MapName
            s.Read(out value.MapName);
            //  Deserialize DirPath
            s.Read(out value.DirPath);
            //  Deserialize array Info
            Rts.Serialization.Reference.Read(s, out value.Info, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.GameSetup.PlayerInfo[] tmp = new Rts.CnC.Messages.GameSetup.PlayerInfo[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_PlayerInfo.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });
            //  Deserialize GameId
            s.Read(out value.GameId);
            //  Deserialize Options
            s.ReadEnum(out value.Options);

            return value;
        }
        
    }
}
